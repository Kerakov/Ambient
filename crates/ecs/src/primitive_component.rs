use elements_std::asset_url::{ObjectAssetType, ObjectRef, TypedAssetUrl};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use paste::paste;
use serde::{Deserialize, Serialize};

use crate::{Component, ComponentRegistry, EntityId, EntityUid, IComponent};

// #[derive(Clone, Serialize, Deserialize, Debug)]
// #[serde(tag = "type")]
// pub enum PrimitiveComponentType {
//     Empty,
//     Bool,
//     EntityId,
//     F32,
//     F64,
//     Mat4,
//     I32,
//     Quat,
//     String,
//     U32,
//     U64,
//     Vec2,
//     Vec3,
//     Vec4,
//     ObjectRef,
//     EntityUid,
//     Vec { variants: Box<PrimitiveComponentType> },
//     Option { variants: Box<PrimitiveComponentType> },
// }
// impl PrimitiveComponentType {
//     pub(crate) fn register(&self, reg: &mut ComponentRegistry, key: &str) {
//         match self {
//             PrimitiveComponentType::Empty => reg.register_with_id(key, &mut Component::<()>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::Bool => reg.register_with_id(key, &mut Component::<bool>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::EntityId => reg.register_with_id(key, &mut Component::<EntityId>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::F32 => reg.register_with_id(key, &mut Component::<f32>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::F64 => reg.register_with_id(key, &mut Component::<f64>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::Mat4 => reg.register_with_id(key, &mut Component::<glam::Mat4>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::I32 => reg.register_with_id(key, &mut Component::<i32>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::Quat => reg.register_with_id(key, &mut Component::<glam::Quat>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::String => reg.register_with_id(key, &mut Component::<String>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::U32 => reg.register_with_id(key, &mut Component::<u32>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::U64 => reg.register_with_id(key, &mut Component::<u64>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::Vec2 => reg.register_with_id(key, &mut Component::<glam::Vec2>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::Vec3 => reg.register_with_id(key, &mut Component::<glam::Vec3>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::Vec4 => reg.register_with_id(key, &mut Component::<glam::Vec4>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::ObjectRef => reg.register_with_id(key, &mut Component::<String>::new_external(0), Some(self.clone())),
//             PrimitiveComponentType::EntityUid => reg.register_with_id(key, &mut Component::<String>::new_external(0), Some(self.clone())),

//             PrimitiveComponentType::Vec { variants } => match **variants {
//                 PrimitiveComponentType::Empty => reg.register_with_id(key, &mut Component::<Vec<()>>::new_external(0), Some(self.clone())),
//                 PrimitiveComponentType::Bool => reg.register_with_id(key, &mut Component::<Vec<bool>>::new_external(0), Some(self.clone())),
//                 PrimitiveComponentType::EntityId => {
//                     reg.register_with_id(key, &mut Component::<Vec<EntityId>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::F32 => reg.register_with_id(key, &mut Component::<Vec<f32>>::new_external(0), Some(self.clone())),
//                 PrimitiveComponentType::F64 => reg.register_with_id(key, &mut Component::<Vec<f64>>::new_external(0), Some(self.clone())),
//                 PrimitiveComponentType::Mat4 => {
//                     reg.register_with_id(key, &mut Component::<Vec<glam::Mat4>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::I32 => reg.register_with_id(key, &mut Component::<Vec<i32>>::new_external(0), Some(self.clone())),
//                 PrimitiveComponentType::Quat => {
//                     reg.register_with_id(key, &mut Component::<Vec<glam::Quat>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::String => {
//                     reg.register_with_id(key, &mut Component::<Vec<String>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::U32 => reg.register_with_id(key, &mut Component::<Vec<u32>>::new_external(0), Some(self.clone())),
//                 PrimitiveComponentType::U64 => reg.register_with_id(key, &mut Component::<Vec<u64>>::new_external(0), Some(self.clone())),
//                 PrimitiveComponentType::Vec2 => {
//                     reg.register_with_id(key, &mut Component::<Vec<glam::Vec2>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::Vec3 => {
//                     reg.register_with_id(key, &mut Component::<Vec<glam::Vec3>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::Vec4 => {
//                     reg.register_with_id(key, &mut Component::<Vec<glam::Vec4>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::ObjectRef => {
//                     reg.register_with_id(key, &mut Component::<Vec<String>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::EntityUid => {
//                     reg.register_with_id(key, &mut Component::<Vec<String>>::new_external(0), Some(self.clone()))
//                 }
//                 _ => panic!("Unsuported Vec inner type: {:?}", variants),
//             },

//             PrimitiveComponentType::Option { variants } => match **variants {
//                 PrimitiveComponentType::Empty => {
//                     reg.register_with_id(key, &mut Component::<Option<()>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::Bool => {
//                     reg.register_with_id(key, &mut Component::<Option<bool>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::EntityId => {
//                     reg.register_with_id(key, &mut Component::<Option<EntityId>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::F32 => {
//                     reg.register_with_id(key, &mut Component::<Option<f32>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::F64 => {
//                     reg.register_with_id(key, &mut Component::<Option<f64>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::Mat4 => {
//                     reg.register_with_id(key, &mut Component::<Option<glam::Mat4>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::I32 => {
//                     reg.register_with_id(key, &mut Component::<Option<i32>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::Quat => {
//                     reg.register_with_id(key, &mut Component::<Option<glam::Quat>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::String => {
//                     reg.register_with_id(key, &mut Component::<Option<String>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::U32 => {
//                     reg.register_with_id(key, &mut Component::<Option<u32>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::U64 => {
//                     reg.register_with_id(key, &mut Component::<Option<u64>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::Vec2 => {
//                     reg.register_with_id(key, &mut Component::<Option<glam::Vec2>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::Vec3 => {
//                     reg.register_with_id(key, &mut Component::<Option<glam::Vec3>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::Vec4 => {
//                     reg.register_with_id(key, &mut Component::<Option<glam::Vec4>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::ObjectRef => {
//                     reg.register_with_id(key, &mut Component::<Option<String>>::new_external(0), Some(self.clone()))
//                 }
//                 PrimitiveComponentType::EntityUid => {
//                     reg.register_with_id(key, &mut Component::<Option<String>>::new_external(0), Some(self.clone()))
//                 }
//                 _ => panic!("Unsuported Option inner type: {:?}", variants),
//             },
//         }
//     }
// }

macro_rules! make_primitive_component {
    ($(($value:ident, $type:ty)),*) => { paste! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum PrimitiveComponent {
            $($value(Component<$type>)), *,
            $([<Vec $value>](Component<Vec<$type>>)), *,
            $([<Option $value>](Component<Option<$type>>)), *
        }

        #[derive(Clone, Serialize, Deserialize, Debug)]
        #[serde(tag = "type")]
        pub enum PrimitiveComponentType {
            $($value), *,
            Vec { variants: Box<PrimitiveComponentType> },
            Option { variants: Box<PrimitiveComponentType> },
        }

        impl PrimitiveComponent {
            fn new(component: &dyn IComponent) -> Option<Self> {
                $(if let Some(comp) = component.downcast_ref::<Component<$type>>() {
                    return Some(PrimitiveComponent::$value(comp.clone()));
                }) *
                $(if let Some(comp) = component.downcast_ref::<Component<Vec<$type>>>() {
                    return Some(PrimitiveComponent::[<Vec $value>](comp.clone()));
                }) *
                $(if let Some(comp) = component.downcast_ref::<Component<Option<$type>>>() {
                    return Some(PrimitiveComponent::[<Option $value>](comp.clone()));
                }) *
                None
            }

            pub fn as_component(&self) -> &dyn IComponent {
                match self {
                  $(Self::$value(c) => c,)*
                  $(Self::[<Vec $value>](c) => c,)*
                  $(Self::[<Option $value>](c) => c,)*
                }
            }
        }
        impl PrimitiveComponentType {
            pub(crate) fn register(&self, reg: &mut ComponentRegistry, key: &str) {
                match self {
                    $(
                        PrimitiveComponentType::$value => reg.register_with_id(key,
                            &mut Component::<$type>::new_external(0),
                            Some(self.clone()),
                            Some(PrimitiveComponent::$value(Component::<$type>::new_external(0)))
                        ),
                    )*
                    PrimitiveComponentType::Vec { variants } => match **variants {
                        $(
                            PrimitiveComponentType::$value => reg.register_with_id(key,
                                &mut Component::<Vec<$type>>::new_external(0),
                                Some(self.clone()),
                                Some(PrimitiveComponent::[<Vec $value>](Component::<Vec<$type>>::new_external(0)))
                            ),
                        )*
                        _ => panic!("Unsuported Vec inner type: {:?}", variants),
                    },
                    PrimitiveComponentType::Option { variants } => match **variants {
                        $(
                            PrimitiveComponentType::$value => reg.register_with_id(key,
                                &mut Component::<Option<$type>>::new_external(0),
                                Some(self.clone()),
                                Some(PrimitiveComponent::[<Option $value>](Component::<Option<$type>>::new_external(0)))
                            ),
                        )*
                        _ => panic!("Unsuported Vec inner type: {:?}", variants),
                    }
                }
            }
        }

        impl PartialEq<PrimitiveComponentType> for PrimitiveComponent {
            fn eq(&self, other: &PrimitiveComponentType) -> bool {
                match (self, other) {
                    $((Self::$value(_), PrimitiveComponentType::$value) => true,)*
                    (pc, PrimitiveComponentType::Vec { variants }) => match (pc, &**variants) {
                        $((Self::[< Vec $value >](_), PrimitiveComponentType::$value) => true,)*
                        _ => false,
                    },
                    (pc, PrimitiveComponentType::Option { variants }) => match (pc, &**variants) {
                        $((Self::[< Option $value >](_), PrimitiveComponentType::$value) => true,)*
                        _ => false,
                    },
                    _ => false,
                }
            }
        }
    } };
}

make_primitive_component!(
    (Empty, ()),
    (Bool, bool),
    (EntityId, EntityId),
    (F32, f32),
    (F64, f64),
    (Mat4, Mat4),
    (I32, i32),
    (Quat, Quat),
    (String, String),
    (U32, u32),
    (U64, u64),
    (Vec2, Vec2),
    (Vec3, Vec3),
    (Vec4, Vec4),
    (ObjectRef, ObjectRef),
    (EntityUid, EntityUid)
);