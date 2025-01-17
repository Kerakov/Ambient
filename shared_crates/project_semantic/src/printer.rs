use crate::{
    Attribute, Component, Concept, Item, ItemId, ItemMap, Message, ResolvableItemId, Scope,
    Semantic, Type, TypeInner,
};

pub struct Printer {
    indent: usize,
}
impl Printer {
    pub fn new() -> Self {
        Self { indent: 0 }
    }

    pub fn print(&mut self, semantic: &Semantic) -> anyhow::Result<()> {
        let items = &semantic.items;
        self.print_scope(items, &*items.get(semantic.root_scope_id)?)?;
        Ok(())
    }

    fn print_scope(&mut self, items: &ItemMap, scope: &Scope) -> anyhow::Result<()> {
        self.print_indent();
        println!(
            "{}",
            fully_qualified_display_path_ambient_style(items, scope, false, true, None)?
        );

        self.with_indent(|p| {
            if let Some(path) = scope.manifest_path.as_deref() {
                p.print_indent();
                println!("path: {}", path.display());
            }

            p.print_indent();
            print!("dependencies: ");
            for (idx, id) in scope.dependencies.iter().copied().enumerate() {
                if idx > 0 {
                    print!(", ");
                }

                print!(
                    "{}",
                    fully_qualified_display_path_ambient_style(
                        items,
                        &*items.get(id)?,
                        false,
                        false,
                        None
                    )?
                );
            }
            println!();

            for id in scope.components.values() {
                p.print_component(items, &*items.get(*id)?)?;
            }

            for id in scope.concepts.values() {
                p.print_concept(items, &*items.get(*id)?)?;
            }

            for id in scope.messages.values() {
                p.print_message(items, &*items.get(*id)?)?;
            }

            for id in scope.types.values() {
                p.print_type(items, &*items.get(*id)?)?;
            }

            for id in scope.attributes.values() {
                p.print_attribute(items, &*items.get(*id)?)?;
            }

            for id in scope.scopes.values() {
                p.print_scope(items, &*items.get(*id)?)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn print_component(&mut self, items: &ItemMap, component: &Component) -> anyhow::Result<()> {
        self.print_indent();
        println!(
            "{}",
            fully_qualified_display_path_ambient_style(items, component, true, true, None)?
        );

        self.with_indent(|p| {
            p.print_indent();
            println!("type: {}", write_resolvable_id(items, &component.type_)?);

            p.print_indent();
            println!("name: {:?}", component.name.as_deref().unwrap_or_default());

            p.print_indent();
            println!(
                "description: {:?}",
                component.description.as_deref().unwrap_or_default()
            );

            p.print_indent();
            println!("default: {:?}", component.default);

            p.print_indent();
            println!("attributes:");
            p.with_indent(|p| {
                for attribute in &component.attributes {
                    p.print_indent();
                    println!("{}", write_resolvable_id(items, attribute)?);
                }
                Ok(())
            })?;

            Ok(())
        })
    }

    fn print_concept(&mut self, items: &ItemMap, concept: &Concept) -> anyhow::Result<()> {
        self.print_indent();
        println!(
            "{}",
            fully_qualified_display_path_ambient_style(items, concept, false, true, None)?
        );

        self.with_indent(|p| {
            p.print_indent();
            println!("name: {:?}", concept.name.as_deref().unwrap_or_default());

            p.print_indent();
            println!(
                "description: {:?}",
                concept.description.as_deref().unwrap_or_default()
            );

            p.print_indent();
            print!("extends:");
            for extend in &concept.extends {
                print!("{} ", write_resolvable_id(items, extend)?);
            }
            println!();

            p.print_indent();
            println!("components:");

            p.with_indent(|p| {
                for (component, value) in concept.components.iter() {
                    p.print_indent();
                    println!("{}: {:?}", write_resolvable_id(items, component)?, value,);
                }

                Ok(())
            })
        })
    }

    fn print_message(&mut self, items: &ItemMap, message: &Message) -> anyhow::Result<()> {
        self.print_indent();
        println!(
            "{}",
            fully_qualified_display_path_ambient_style(items, message, false, true, None)?
        );

        self.with_indent(|p| {
            p.print_indent();
            println!(
                "description: {:?}",
                message.description.as_deref().unwrap_or_default()
            );

            p.print_indent();
            println!("fields:");

            p.with_indent(|p| {
                for (id, ty) in message.fields.iter() {
                    p.print_indent();
                    println!("{}: {}", id, write_resolvable_id(items, ty)?);
                }

                Ok(())
            })
        })
    }

    fn print_type(&mut self, items: &ItemMap, type_: &Type) -> anyhow::Result<()> {
        self.print_indent();
        println!(
            "{}",
            fully_qualified_display_path_ambient_style(items, type_, false, true, None)?,
        );
        if let TypeInner::Enum(e) = &type_.inner {
            self.with_indent(|p| {
                p.print_indent();
                println!(
                    "description: {:?}",
                    e.description.as_deref().unwrap_or_default()
                );

                p.print_indent();
                println!(
                    "members: {:?}",
                    e.description.as_deref().unwrap_or_default()
                );
                p.with_indent(|p| {
                    for (name, description) in &e.members {
                        p.print_indent();
                        print!("{name}: {description:?}");
                        println!();
                    }
                    Ok(())
                })?;
                Ok(())
            })?;
        }
        Ok(())
    }

    fn print_attribute(&mut self, items: &ItemMap, attribute: &Attribute) -> anyhow::Result<()> {
        self.print_indent();
        println!(
            "{}",
            fully_qualified_display_path_ambient_style(items, attribute, false, true, None)?
        );
        Ok(())
    }

    fn print_indent(&self) {
        for _ in 0..self.indent {
            print!("  ");
        }
    }

    fn with_indent(
        &mut self,
        f: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.indent += 1;
        f(self)?;
        self.indent -= 1;
        Ok(())
    }
}

fn write_resolvable_id<T: Item>(
    items: &ItemMap,
    r: &ResolvableItemId<T>,
) -> anyhow::Result<String> {
    Ok(match r {
        ResolvableItemId::Unresolved(unresolved) => format!("unresolved({:?})", unresolved),
        ResolvableItemId::Resolved(resolved) => fully_qualified_display_path_ambient_style(
            items,
            &*items.get(*resolved)?,
            true,
            true,
            None,
        )?,
    })
}

pub fn fully_qualified_display_path_ambient_style<T: Item>(
    items: &ItemMap,
    item: &T,
    use_original_scope_ids: bool,
    display_affixes: bool,
    relative_to: Option<ItemId<Scope>>,
) -> anyhow::Result<String> {
    items.fully_qualified_display_path_impl(
        item,
        "::",
        use_original_scope_ids,
        (display_affixes, display_affixes),
        relative_to,
        None,
    )
}
