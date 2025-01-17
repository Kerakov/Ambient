use std::{path::PathBuf, str::FromStr};

use ambient_audio::AudioStream;
use ambient_core::window::ExitStatus;
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, ContentBaseUrlKey},
    download_asset::{AssetsCacheOnDisk, ReqwestClientKey},
};
use ambient_network::native::client::ResolvedAddr;
use clap::Parser;

mod cli;
mod client;
mod server;
mod shared;

use ambient_physics::physx::PhysicsKey;
use anyhow::{bail, Context};
use cli::{AssetCommand, Cli, Commands};
use log::LevelFilter;
use server::QUIC_INTERFACE_PORT;

#[cfg(not(feature = "no_bundled_certs"))]
const CERT: &[u8] = include_bytes!("../../localhost.crt");

#[cfg(not(feature = "no_bundled_certs"))]
const CERT_KEY: &[u8] = include_bytes!("../../localhost.key");

fn main() -> anyhow::Result<()> {
    let rt = ambient_sys::task::make_native_multithreaded_runtime()?;

    setup_logging()?;

    shared::components::init()?;

    let runtime = rt.handle();
    let assets = AssetCache::new(runtime.clone());
    PhysicsKey.get(&assets); // Load physics
    AssetsCacheOnDisk.insert(&assets, false); // Disable disk caching for now; see https://github.com/AmbientRun/Ambient/issues/81

    let cli = Cli::parse();

    let project = cli.project();

    if let Some(project) = project {
        if project.project {
            log::warn!("`-p`/`--project` has no semantic meaning - the path is always treated as a project path.");
            log::warn!("You do not need to use `-p`/`--project` - `ambient run project` is the same as `ambient run -p project`.");
        }
    }

    let project_path: ProjectPath = project.and_then(|p| p.path.clone()).try_into()?;

    if project_path.is_remote() {
        // project path is a URL, so let's use it as the content base URL
        ContentBaseUrlKey.insert(&assets, project_path.url.push("build/")?);
    }

    // If new: create project, immediately exit
    if let Commands::New { name, api_path, .. } = &cli.command {
        if let Some(path) = &project_path.fs_path {
            if let Err(err) =
                cli::new_project::new_project(path, name.as_deref(), api_path.as_deref())
            {
                eprintln!("Failed to create project: {err:?}");
            }
        } else {
            eprintln!("Cannot create project in a remote directory.");
        }
        return Ok(());
    }

    if let Commands::Assets { command } = &cli.command {
        return rt.block_on(async {
            match command {
                AssetCommand::MigratePipelinesToml(opt) => {
                    let path = ProjectPath::new_local(opt.path.clone())?;
                    ambient_build::migrate::toml::process(path.fs_path.unwrap())
                        .await
                        .context("Failed to migrate pipelines")?;
                }
                AssetCommand::Import(opt) => match opt.path.extension() {
                    Some(ext) => {
                        if ext == "wav" || ext == "mp3" || ext == "ogg" {
                            let convert = opt.convert_audio;
                            ambient_build::pipelines::import_audio(opt.path.clone(), convert)
                                .context("failed to import audio")?;
                        } else if ext == "fbx" || ext == "glb" || ext == "gltf" || ext == "obj" {
                            let collider_from_model = opt.collider_from_model;
                            ambient_build::pipelines::import_model(
                                opt.path.clone(),
                                collider_from_model,
                            )
                            .context("failed to import models")?;
                        } else if ext == "jpg" || ext == "png" || ext == "gif" || ext == "webp" {
                            // TODO: import textures API may change, so this is just a placeholder
                            todo!();
                        } else {
                            bail!("Unsupported file type");
                        }
                    }
                    None => bail!("Unknown file type"),
                },
            }

            Ok(())
        });
    }

    // Build the project if required. Note that this only runs if the project is local.
    //
    // Update the project path to match the build path if necessary.
    let original_project_path = project_path.clone();
    let (project_path, build_path) = if let Some((project, project_path)) = project
        .as_ref()
        .filter(|p| !p.no_build)
        .zip(project_path.fs_path.as_deref())
    {
        rt.block_on(async {
            let build_path = project_path.join("build");
            // The build step uses its own semantic to ensure that there is
            // no contamination, so that the built project can use its own
            // semantic based on the flat hierarchy.
            let mut semantic = ambient_project_semantic::Semantic::new().await?;
            let primary_ember_scope_id =
                shared::ember::add(None, &mut semantic, project_path).await?;

            let manifest = semantic
                .items
                .get(primary_ember_scope_id)?
                .manifest
                .clone()
                .context("no manifest for scope")?;

            let build_config = ambient_build::BuildConfiguration {
                build_path: build_path.clone(),
                assets: assets.clone(),
                semantic: &mut semantic,
                optimize: project.release,
                clean_build: project.clean_build,
                build_wasm_only: project.build_wasm_only,
            };

            let project_name = manifest
                .ember
                .name
                .as_deref()
                .unwrap_or_else(|| manifest.ember.id.as_str());

            tracing::info!("Building project {:?}", project_name);

            let output_path = ambient_build::build(build_config, primary_ember_scope_id)
                .await
                .context("Failed to build project")?;

            anyhow::Ok((
                ProjectPath::new_local(output_path)?,
                Some(AbsAssetUrl::from_file_path(build_path)),
            ))
        })?
    } else {
        (project_path.clone(), None)
    };

    // If this is just a build, exit now
    if matches!(&cli.command, Commands::Build { .. }) {
        return Ok(());
    }

    // Read the project manifest from the project path (which may have been updated by the build step)
    // We attempt both the root and build/ as `ambient.toml` is in the former for local builds,
    // and in the latter for deployed builds. This will likely be improved if/when deployments
    // no longer have their own build directory.
    async fn get_new_project_path_and_manifest(
        project_path: ProjectPath,
        assets: &AssetCache,
    ) -> anyhow::Result<(ProjectPath, ambient_project::Manifest)> {
        let paths = [project_path.url.clone(), project_path.push("build")];

        for path in &paths {
            if let Ok(toml) = path.push("ambient.toml")?.download_string(assets).await {
                return Ok((
                    Some(path.to_string()).try_into()?,
                    ambient_project::Manifest::parse(&toml)?,
                ));
            }
        }

        anyhow::bail!("Failed to find ambient.toml in project");
    }

    let (project_path, manifest) =
        rt.block_on(get_new_project_path_and_manifest(project_path, &assets))?;
    let build_path = build_path.unwrap_or_else(|| project_path.url.push("build").unwrap());

    // If this is just a deploy then deploy and exit
    if let Commands::Deploy {
        token,
        api_server,
        force_upload,
        ensure_running,
        context,
        ..
    } = &cli.command
    {
        return rt.block_on(async {
            let Some(project_fs_path) = &project_path.fs_path else {
                anyhow::bail!("Can only deploy a local project");
            };
            let deployment_id = ambient_deploy::deploy(
                runtime,
                api_server,
                token,
                project_fs_path,
                &manifest,
                *force_upload,
            )
            .await?;
            log::info!(
                "Assets deployed successfully. Deployment id: {}. Deploy url: https://assets.ambient.run/{}",
                deployment_id,
                deployment_id,
            );
            if *ensure_running {
                let spec = ambient_cloud_client::ServerSpec::new_with_deployment(deployment_id)
                    .with_context(context.clone());
                let server = ambient_cloud_client::ensure_server_running(
                    &assets,
                    api_server,
                    token.into(),
                    spec,
                )
                .await?;
                log::info!("Deployed ember is running at {}", server.host);
            }
            Ok(())
        });
    }

    // Otherwise, either connect to a server or host one
    let server_addr = if let Commands::Join { host, .. } = &cli.command {
        if let Some(mut host) = host.clone() {
            rt.block_on(async {
                if host.starts_with("http://") || host.starts_with("https://") {
                    tracing::info!("NOTE: Joining server by http url is still experimental and can be removed without warning.");

                    host = ReqwestClientKey
                        .get(&assets)
                        .get(host)
                        .send()
                        .await?
                        .text()
                        .await?;
                    if host.is_empty() {
                        anyhow::bail!("Failed to resolve host");
                    }
                }
                if !host.contains(':') {
                    host = format!("{host}:{QUIC_INTERFACE_PORT}");
                }
                ResolvedAddr::lookup_host(&host).await
            })?
        } else {
            ResolvedAddr::localhost_with_port(QUIC_INTERFACE_PORT)
        }
    } else if let Some(host) = &cli.host() {
        let crypto = if let (Some(cert_file), Some(key_file)) = (&host.cert, &host.key) {
            let raw_cert = std::fs::read(cert_file).context("Failed to read certificate file")?;
            let cert_chain = if raw_cert.starts_with(b"-----BEGIN CERTIFICATE-----") {
                rustls_pemfile::certs(&mut raw_cert.as_slice())
                    .context("Failed to parse certificate file")?
            } else {
                vec![raw_cert]
            };
            let raw_key = std::fs::read(key_file).context("Failed to read certificate key")?;
            let key = if raw_key.starts_with(b"-----BEGIN ") {
                rustls_pemfile::read_all(&mut raw_key.as_slice())
                    .context("Failed to parse certificate key")?
                    .into_iter()
                    .find_map(|item| match item {
                        rustls_pemfile::Item::RSAKey(key) => Some(key),
                        rustls_pemfile::Item::PKCS8Key(key) => Some(key),
                        rustls_pemfile::Item::ECKey(key) => Some(key),
                        _ => None,
                    })
                    .ok_or_else(|| anyhow::anyhow!("No private key found"))?
            } else {
                raw_key
            };
            ambient_network::native::server::Crypto { cert_chain, key }
        } else {
            #[cfg(feature = "no_bundled_certs")]
            {
                anyhow::bail!("--cert and --key are required without bundled certs.");
            }
            #[cfg(not(feature = "no_bundled_certs"))]
            {
                tracing::info!("Using bundled certificate and key");
                ambient_network::native::server::Crypto {
                    cert_chain: vec![CERT.to_vec()],
                    key: CERT_KEY.to_vec(),
                }
            }
        };

        let working_directory = project_path
            .fs_path
            .clone()
            .unwrap_or(std::env::current_dir()?);

        let addr = rt.block_on(server::start(
            runtime,
            assets.clone(),
            cli.clone(),
            working_directory,
            project_path.url.clone(),
            build_path,
            manifest,
            crypto,
        ));

        ResolvedAddr::localhost_with_port(addr.port())
    } else {
        unreachable!()
    };

    // Time to join!
    if let Some(run) = cli.run() {
        // Hey! listen, it is time to setup audio

        let audio_stream = if !run.mute_audio {
            log::info!("Creating audio stream");
            match AudioStream::new().context("Failed to initialize audio stream") {
                Ok(v) => Some(v),
                Err(err) => {
                    log::error!("Failed to initialize audio stream: {err}");
                    None
                }
            }
        } else {
            log::info!("Audio is disabled");
            None
        };

        let mixer = if run.mute_audio {
            None
        } else {
            audio_stream.as_ref().map(|v| v.mixer().clone())
        };

        // If we have run parameters, start a client and join a server
        let exit_status = rt.block_on(client::run(
            assets,
            server_addr,
            run,
            original_project_path.fs_path,
            mixer,
        ));
        if exit_status == ExitStatus::FAILURE {
            bail!("client::run failed with {exit_status:?}");
        }
    } else {
        // Otherwise, wait for the Ctrl+C signal
        match rt.block_on(tokio::signal::ctrl_c()) {
            Ok(()) => {}
            Err(err) => log::error!("Unable to listen for shutdown signal: {}", err),
        }
    }

    Ok(())
}

fn setup_logging() -> anyhow::Result<()> {
    const MODULES: &[(LevelFilter, &[&str])] = &[
        (
            LevelFilter::Error,
            &[
                // Warns about extra syntactic elements; we are not concerned with these.
                "fbxcel",
            ],
        ),
        (
            LevelFilter::Warn,
            &[
                "ambient_gpu",
                "ambient_model",
                "ambient_physics",
                "ambient_native_std",
                "cranelift_codegen",
                "naga",
                "tracing",
                "symphonia_core",
                "symphonia_bundle_mp3",
                "wgpu_core",
                "wgpu_hal",
                "optivorbis",
                "symphonia_format_wav",
            ],
        ),
    ];

    // Initialize the logger and lower the log level for modules we don't need to hear from by default.
    #[cfg(not(feature = "tracing"))]
    {
        let mut builder = env_logger::builder();
        builder.filter_level(LevelFilter::Info);

        for (level, modules) in MODULES {
            for module in *modules {
                builder.filter_module(module, *level);
            }
        }

        builder.parse_default_env().try_init()?;

        Ok(())
    }

    #[cfg(feature = "tracing")]
    {
        use tracing::metadata::Level;
        use tracing_log::AsTrace;
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::{registry, EnvFilter};

        let mut filter = tracing_subscriber::filter::Targets::new()
            .with_default(tracing::metadata::LevelFilter::DEBUG);
        for (level, modules) in MODULES {
            for &module in *modules {
                filter = filter.with_target(module, level.as_trace());
            }
        }

        // BLOCKING: pending https://github.com/tokio-rs/tracing/issues/2507
        // let modules: Vec<_> = MODULES.iter().flat_map(|&(level, modules)| modules.iter().map(move |&v| format!("{v}={level}"))).collect();

        // eprintln!("{modules:#?}");
        // let mut filter = tracing_subscriber::filter::EnvFilter::builder().with_default_directive(Level::INFO.into()).from_env_lossy();

        // for module in modules {
        //     filter = filter.add_directive(module.parse().unwrap());
        // }

        // let mut filter = std::env::var("RUST_LOG").unwrap_or_default().parse::<tracing_subscriber::filter::Targets>().unwrap_or_default();
        // filter.extend(MODULES.iter().flat_map(|&(level, modules)| modules.iter().map(move |&v| (v, level.as_trace()))));

        let env_filter = EnvFilter::builder()
            .with_default_directive(Level::INFO.into())
            .from_env_lossy();

        let layered_registry = registry().with(filter).with(env_filter);

        // use stackdriver format if available and requested
        #[cfg(feature = "stackdriver")]
        if std::env::var("LOG_FORMAT").unwrap_or_default() == "stackdriver" {
            layered_registry
                .with(tracing_stackdriver::layer().with_writer(std::io::stdout))
                .try_init()?;
            return Ok(());
        }

        // otherwise use the default format
        layered_registry
            .with(tracing_subscriber::fmt::Layer::new().with_timer(
                tracing_subscriber::fmt::time::LocalTime::new(time::macros::format_description!(
                    "[hour]:[minute]:[second]"
                )),
            ))
            .try_init()?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ProjectPath {
    url: AbsAssetUrl,
    fs_path: Option<std::path::PathBuf>,
}

impl ProjectPath {
    fn new_local(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let current_dir = std::env::current_dir().context("Error getting current directory")?;
        let path = if path.is_absolute() {
            path
        } else {
            ambient_std::path::normalize(&current_dir.join(path))
        };

        if path.exists() && !path.is_dir() {
            anyhow::bail!("Project path {path:?} exists and is not a directory.");
        }
        let url = AbsAssetUrl::from_directory_path(path);
        let fs_path = url.to_file_path().ok().flatten();

        Ok(Self { url, fs_path })
    }

    fn is_remote(&self) -> bool {
        self.fs_path.is_none()
    }

    // 'static to limit only to compile-time known paths
    fn push(&self, path: &'static str) -> AbsAssetUrl {
        self.url.push(path).unwrap()
    }
}

impl TryFrom<Option<String>> for ProjectPath {
    type Error = anyhow::Error;

    fn try_from(project_path: Option<String>) -> anyhow::Result<Self> {
        match project_path {
            Some(project_path)
                if project_path.starts_with("http://")
                    || project_path.starts_with("https://")
                    || project_path.starts_with("file:/") =>
            {
                let url = AbsAssetUrl::from_str(&project_path)?;
                if let Some(local) = url.to_file_path()? {
                    Self::new_local(local)
                } else {
                    Ok(Self { url, fs_path: None })
                }
            }
            Some(project_path) => Self::new_local(project_path),
            None => {
                let url = AbsAssetUrl::from_directory_path(std::env::current_dir()?);
                let fs_path = url.to_file_path().ok().flatten();
                Ok(Self { url, fs_path })
            }
        }
    }
}
