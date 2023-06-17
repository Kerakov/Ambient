use std::path::PathBuf;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Component, Concept, Enum, Identifier, ItemPathBuf, Message, Version};

#[derive(Error, Debug, PartialEq)]
pub enum ManifestParseError {
    #[error("manifest was not valid TOML")]
    TomlError(#[from] toml::de::Error),
    #[error("manifest contains a project section; projects have been renamed to embers")]
    ProjectRenamedToEmberError,
}

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Serialize)]
pub struct Manifest {
    #[serde(default)]
    pub ember: Ember,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    #[serde(alias = "component")]
    pub components: IndexMap<ItemPathBuf, Component>,
    #[serde(default)]
    #[serde(alias = "concept")]
    pub concepts: IndexMap<ItemPathBuf, Concept>,
    #[serde(default)]
    #[serde(alias = "message")]
    pub messages: IndexMap<ItemPathBuf, Message>,
    #[serde(default)]
    #[serde(alias = "enum")]
    pub enums: IndexMap<Identifier, Enum>,
    #[serde(default)]
    pub dependencies: IndexMap<ItemPathBuf, Dependency>,
}
impl Manifest {
    pub fn parse(manifest: &str) -> Result<Self, ManifestParseError> {
        let raw = toml::from_str::<toml::Table>(manifest)?;
        if raw.contains_key("project") {
            return Err(ManifestParseError::ProjectRenamedToEmberError);
        }

        Ok(toml::from_str(manifest)?)
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Ember {
    pub id: Identifier,
    pub organization: Option<Identifier>,
    pub name: Option<String>,
    pub version: Option<Version>,
    pub description: Option<String>,
    pub repository: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default, rename = "type")]
    pub type_: EmberType,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub includes: Vec<PathBuf>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize, Default)]
pub enum EmberType {
    #[default]
    Game,
    Mod,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub enum Category {
    Example,
    Fps,
    Survival,
    Simulation,
    Multiplayer,
    Strategy,
    Sports,
    Racing,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Build {
    #[serde(default)]
    pub rust: BuildRust,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct BuildRust {
    #[serde(rename = "feature-multibuild")]
    pub feature_multibuild: Vec<String>,
}
impl Default for BuildRust {
    fn default() -> Self {
        Self {
            feature_multibuild: vec!["client".to_string(), "server".to_string()],
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Dependency {
    Path { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use indexmap::IndexMap;

    use crate::{
        Build, BuildRust, Component, ComponentType, Concept, ContainerType, Dependency, Ember,
        Enum, Identifier, ItemPathBuf, Manifest, ManifestParseError, Version, VersionSuffix,
    };

    fn i(s: &str) -> Identifier {
        Identifier::new(s).unwrap()
    }

    fn ipb(s: &str) -> ItemPathBuf {
        ItemPathBuf::new(s).unwrap()
    }

    #[test]
    fn can_parse_minimal_toml() {
        const TOML: &str = r#"
        [ember]
        id = "test"
        name = "Test"
        version = "0.0.1"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: Identifier::new("test").unwrap(),
                    name: Some("Test".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    ..Default::default()
                },
                ..Default::default()
            })
        );
    }

    #[test]
    fn will_fail_on_legacy_project_toml() {
        const TOML: &str = r#"
        [project]
        id = "test"
        name = "Test"
        version = "0.0.1"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Err(ManifestParseError::ProjectRenamedToEmberError)
        )
    }

    #[test]
    fn can_parse_tictactoe_toml() {
        const TOML: &str = r#"
        [ember]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [components]
        cell = { type = "i32", name = "Cell", description = "The ID of the cell this player is in", attributes = ["store"] }

        [concepts.cell]
        name = "Cell"
        description = "A cell object"
        [concepts.cell.components]
        cell = 0
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: i("tictactoe"),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([(
                    ipb("cell"),
                    Component {
                        name: Some("Cell".to_string()),
                        description: Some("The ID of the cell this player is in".to_string()),
                        type_: ComponentType::Item(i("i32").into()),
                        attributes: vec![i("store").into()],
                        default: None,
                    }
                    .into()
                )]),
                concepts: IndexMap::from_iter([(
                    ipb("cell"),
                    Concept {
                        name: Some("Cell".to_string()),
                        description: Some("A cell object".to_string()),
                        extends: vec![],
                        components: IndexMap::from_iter([(ipb("cell"), toml::Value::Integer(0))])
                    }
                    .into()
                )]),
                messages: Default::default(),
                enums: Default::default(),
                dependencies: Default::default(),
            })
        )
    }

    #[test]
    fn can_parse_rust_build_settings() {
        const TOML: &str = r#"
        [ember]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [build.rust]
        feature-multibuild = ["client"]
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: i("tictactoe"),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string()]
                    }
                },
                ..Default::default()
            })
        )
    }

    #[test]
    fn can_parse_concepts_with_documented_namespace_from_manifest() {
        use toml::Value;

        const TOML: &str = r#"
        [ember]
        id = "my-project"
        name = "My Project"
        version = "0.0.1"

        [components]
        "core/transform/rotation" = { type = "quat", name = "Rotation", description = "" }
        "core/transform/scale" = { type = "vec3", name = "Scale", description = "" }
        "core/transform/spherical-billboard" = { type = "empty", name = "Spherical billboard", description = "" }
        "core/transform/translation" = { type = "vec3", name = "Translation", description = "" }

        [concepts."ns/transformable"]
        name = "Transformable"
        description = "Can be translated, rotated and scaled."

        [concepts."ns/transformable".components]
        # This is intentionally out of order to ensure that order is preserved
        "core/transform/translation" = [0, 0, 0]
        "core/transform/scale" = [1, 1, 1]
        "core/transform/rotation" = [0, 0, 0, 1]
        "#;

        let manifest = Manifest::parse(TOML).unwrap();
        assert_eq!(
            manifest,
            Manifest {
                ember: Ember {
                    id: i("my-project"),
                    name: Some("My Project".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([
                    (
                        ipb("core/transform/rotation"),
                        Component {
                            name: Some("Rotation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("quat").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ipb("core/transform/scale"),
                        Component {
                            name: Some("Scale".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("vec3").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ipb("core/transform/spherical-billboard"),
                        Component {
                            name: Some("Spherical billboard".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("empty").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ipb("core/transform/translation"),
                        Component {
                            name: Some("Translation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("vec3").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                ]),
                concepts: IndexMap::from_iter([(
                    ipb("ns/transformable"),
                    Concept {
                        name: Some("Transformable".to_string()),
                        description: Some("Can be translated, rotated and scaled.".to_string()),
                        extends: vec![],
                        components: IndexMap::from_iter([
                            (
                                ipb("core/transform/translation"),
                                Value::Array(vec![
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(0)
                                ])
                            ),
                            (
                                ipb("core/transform/scale"),
                                Value::Array(vec![
                                    Value::Integer(1),
                                    Value::Integer(1),
                                    Value::Integer(1)
                                ])
                            ),
                            (
                                ipb("core/transform/rotation"),
                                Value::Array(vec![
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(1)
                                ])
                            ),
                        ])
                    }
                    .into()
                )]),
                messages: Default::default(),
                enums: Default::default(),
                dependencies: Default::default(),
            }
        );

        assert_eq!(
            manifest
                .concepts
                .first()
                .unwrap()
                .1
                .components
                .keys()
                .collect::<Vec<_>>(),
            vec![
                &ipb("core/transform/translation"),
                &ipb("core/transform/scale"),
                &ipb("core/transform/rotation"),
            ]
        );
    }

    #[test]
    fn can_parse_enums() {
        const TOML: &str = r#"
        [ember]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [enums.cell-state]
        taken = "The cell is taken"
        free = "The cell is free"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: i("tictactoe"),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    ..Default::default()
                },
                build: Build::default(),
                components: Default::default(),
                concepts: Default::default(),
                messages: Default::default(),
                enums: IndexMap::from_iter([(
                    i("cell-state"),
                    Enum(IndexMap::from_iter([
                        (i("taken"), "The cell is taken".to_string()),
                        (i("free"), "The cell is free".to_string()),
                    ]))
                    .into()
                )]),
                dependencies: Default::default(),
            })
        )
    }

    #[test]
    fn can_parse_container_types() {
        const TOML: &str = r#"
        [ember]
        id = "test"
        name = "Test"
        version = "0.0.1"

        [components]
        test = { type = "i32", name = "Test", description = "Test" }
        vec-test = { type = { container_type = "vec", element_type = "i32" }, name = "Test", description = "Test" }
        option-test = { type = { container_type = "option", element_type = "i32" }, name = "Test", description = "Test" }

        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: i("test"),
                    name: Some("Test".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([
                    (
                        ipb("test"),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Item(i("i32").into()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ipb("vec-test"),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Contained {
                                type_: ContainerType::Vec,
                                element_type: i("i32").into()
                            },
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        ipb("option-test"),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Contained {
                                type_: ContainerType::Option,
                                element_type: i("i32").into()
                            },
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    )
                ]),
                concepts: Default::default(),
                messages: Default::default(),
                enums: Default::default(),
                dependencies: Default::default(),
            })
        )
    }

    #[test]
    fn can_parse_dependencies() {
        const TOML: &str = r#"
        [ember]
        id = "dependencies"
        organization = "ambient"
        name = "dependencies"
        version = "0.0.1"

        [dependencies]
        "ambient/deps-assets" = { path = "deps/assets" }
        "ambient/deps-code" = { path = "deps/code" }

        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: i("dependencies"),
                    organization: Some(i("ambient")),
                    name: Some("dependencies".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    ..Default::default()
                },
                build: Default::default(),
                components: Default::default(),
                concepts: Default::default(),
                messages: Default::default(),
                enums: Default::default(),
                dependencies: IndexMap::from_iter([
                    (
                        ipb("ambient/deps-assets"),
                        Dependency::Path {
                            path: PathBuf::from("deps/assets")
                        }
                    ),
                    (
                        ipb("ambient/deps-code"),
                        Dependency::Path {
                            path: PathBuf::from("deps/code")
                        }
                    )
                ])
            })
        )
    }
}
