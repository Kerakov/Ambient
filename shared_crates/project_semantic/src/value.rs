use crate::{ItemId, ItemMap, TypeInner};

use super::{PrimitiveType, Type};
use ambient_project::PascalCaseIdentifier;
use ambient_shared_types::{
    primitive_component_definitions, ProceduralMaterialHandle, ProceduralMeshHandle,
    ProceduralSamplerHandle, ProceduralTextureHandle,
};
use anyhow::Context as AnyhowContext;
use glam::{IVec2, IVec3, IVec4, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use std::{fmt, time::Duration};

pub type EntityId = u128;

#[derive(Clone, PartialEq, Debug)]
pub enum ResolvableValue {
    Unresolved(toml::Value),
    Resolved(Value),
}
impl ResolvableValue {
    pub(crate) fn resolve(&mut self, items: &ItemMap, id: ItemId<Type>) -> anyhow::Result<()> {
        if let Self::Unresolved(value) = self {
            *self = Self::Resolved(Value::from_toml(value, items, id)?);
        }
        Ok(())
    }

    pub fn as_resolved(&self) -> Option<&Value> {
        match self {
            Self::Resolved(value) => Some(value),
            _ => None,
        }
    }
}

macro_rules! define_scalar_value {
    ($(($value:ident, $type:ty)),*) => {
        paste::paste! {
            #[derive(Debug, Clone, PartialEq)]
            pub enum ScalarValue {
                $(
                    $value($type),
                )*
            }
            $(
                impl From<$type> for ScalarValue {
                    fn from(value: $type) -> Self {
                        Self::$value(value)
                    }
                }
            )*
            impl fmt::Display for ScalarValue {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    match self {
                        $(
                            Self::$value(value) => fmt::Debug::fmt(value, f),
                        )*
                    }
                }
            }
        }
    };
}
primitive_component_definitions!(define_scalar_value);

impl ScalarValue {
    pub fn from_toml(value: &toml::Value, ty: PrimitiveType) -> anyhow::Result<Self> {
        fn as_bool(v: &toml::Value) -> anyhow::Result<bool> {
            v.as_bool()
                .with_context(|| format!("Expected bool, got {:?}", v))
        }

        fn as_integer(v: &toml::Value) -> anyhow::Result<i64> {
            v.as_integer()
                .with_context(|| format!("Expected integer, got {:?}", v))
        }

        fn as_str(v: &toml::Value) -> anyhow::Result<&str> {
            v.as_str()
                .with_context(|| format!("Expected string, got {:?}", v))
        }

        fn as_array<T: Default + Copy, const N: usize>(
            value: &toml::Value,
            converter: impl Fn(&toml::Value) -> anyhow::Result<T>,
        ) -> anyhow::Result<[T; N]> {
            let arr = value
                .as_array()
                .with_context(|| format!("Expected array, got {:?}", value))?;

            assert_eq!(arr.len(), N);

            let mut result = [T::default(); N];
            for (i, v) in arr.iter().enumerate() {
                result[i] = converter(v)?;
            }
            Ok(result)
        }

        fn as_float(v: &toml::Value) -> anyhow::Result<f64> {
            v.as_float()
                .with_context(|| format!("Expected float, got {:?}", v))
        }

        let v = value;
        Ok(match ty {
            PrimitiveType::Empty => Self::Empty(()),
            PrimitiveType::Bool => Self::Bool(as_bool(v)?),
            PrimitiveType::EntityId => {
                let value = as_str(v)?;
                let bytes = data_encoding::BASE64
                    .decode(value.as_bytes())
                    .with_context(|| format!("Failed to decode Base64 for entity id {value:?}"))?;
                let bytes = bytes.as_slice().try_into().with_context(|| {
                    format!("Failed to convert decoded Base64 bytes {bytes:?} to entity id")
                })?;
                Self::EntityId(EntityId::from_le_bytes(bytes))
            }
            PrimitiveType::F32 => Self::F32(as_float(v)? as f32),
            PrimitiveType::F64 => Self::F64(as_float(v)?),
            PrimitiveType::Mat4 => Self::Mat4(Mat4::from_cols_array(&as_array(v, |v| {
                Ok(as_float(v)? as f32)
            })?)),
            PrimitiveType::Quat => {
                Self::Quat(Quat::from_array(as_array(v, |v| Ok(as_float(v)? as f32))?))
            }
            PrimitiveType::String => Self::String(as_str(v)?.to_string()),
            PrimitiveType::U8 => Self::U8(as_integer(v)? as u8),
            PrimitiveType::U16 => Self::U16(as_integer(v)? as u16),
            PrimitiveType::U32 => Self::U32(as_integer(v)? as u32),
            PrimitiveType::U64 => Self::U64(as_integer(v)? as u64),
            PrimitiveType::I8 => Self::I8(as_integer(v)? as i8),
            PrimitiveType::I16 => Self::I16(as_integer(v)? as i16),
            PrimitiveType::I32 => Self::I32(as_integer(v)? as i32),
            PrimitiveType::I64 => Self::I64(as_integer(v)?),
            PrimitiveType::Vec2 => {
                Self::Vec2(Vec2::from_array(as_array(v, |v| Ok(as_float(v)? as f32))?))
            }
            PrimitiveType::Vec3 => {
                Self::Vec3(Vec3::from_array(as_array(v, |v| Ok(as_float(v)? as f32))?))
            }
            PrimitiveType::Vec4 => {
                Self::Vec4(Vec4::from_array(as_array(v, |v| Ok(as_float(v)? as f32))?))
            }
            PrimitiveType::Uvec2 => Self::Uvec2(UVec2::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as u32)
            })?)),
            PrimitiveType::Uvec3 => Self::Uvec3(UVec3::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as u32)
            })?)),
            PrimitiveType::Uvec4 => Self::Uvec4(UVec4::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as u32)
            })?)),
            PrimitiveType::Ivec2 => Self::Ivec2(IVec2::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as i32)
            })?)),
            PrimitiveType::Ivec3 => Self::Ivec3(IVec3::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as i32)
            })?)),
            PrimitiveType::Ivec4 => Self::Ivec4(IVec4::from_array(as_array(v, |v| {
                Ok(as_integer(v)? as i32)
            })?)),
            PrimitiveType::Duration => anyhow::bail!("unsupported value to load from TOML"),
            PrimitiveType::ProceduralMeshHandle => {
                anyhow::bail!("unsupported value to load from TOML")
            }
            PrimitiveType::ProceduralTextureHandle => {
                anyhow::bail!("unsupported value to load from TOML")
            }
            PrimitiveType::ProceduralSamplerHandle => {
                anyhow::bail!("unsupported value to load from TOML")
            }
            PrimitiveType::ProceduralMaterialHandle => {
                anyhow::bail!("unsupported value to load from TOML")
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Scalar(ScalarValue),
    Vec(Vec<ScalarValue>),
    Option(Option<ScalarValue>),
    Enum(ItemId<Type>, PascalCaseIdentifier),
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(v) => fmt::Display::fmt(v, f),
            Self::Vec(v) => {
                write!(f, "[")?;
                for (i, v) in v.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    fmt::Display::fmt(v, f)?;
                }
                write!(f, "]")
            }
            Self::Option(v) => {
                if let Some(v) = v {
                    write!(f, "Some({})", v)
                } else {
                    write!(f, "None")
                }
            }
            Self::Enum(ty, v) => write!(f, "{ty}::{v}"),
        }
    }
}
impl Value {
    pub(crate) fn from_toml(
        value: &toml::Value,
        items: &ItemMap,
        ty_id: ItemId<Type>,
    ) -> anyhow::Result<Self> {
        let ty = &*items.get(ty_id)?;
        Ok(match &ty.inner {
            TypeInner::Primitive(pt) => Self::Scalar(ScalarValue::from_toml(value, *pt)?),
            TypeInner::Vec(v) => {
                let ty = &*items.get(*v)?;
                let inner_ty_id = ty
                    .inner
                    .as_vec()
                    .with_context(|| format!("Expected vector type, got {:?}", ty))?;
                let inner_ty = &*items.get(inner_ty_id)?;
                let inner_ty = inner_ty.inner.as_primitive().with_context(|| {
                    format!("Expected primitive type, got {:?}", inner_ty.inner)
                })?;

                let arr = value
                    .as_array()
                    .with_context(|| format!("Expected array, got {:?}", value))?;

                Self::Vec(
                    arr.iter()
                        .map(|v| ScalarValue::from_toml(v, inner_ty))
                        .collect::<anyhow::Result<_>>()?,
                )
            }
            TypeInner::Option(o) => {
                let ty = &*items.get(*o)?;
                let inner_ty = ty
                    .inner
                    .as_option()
                    .with_context(|| format!("Expected option type, got {:?}", ty))?;
                let inner_ty = &*items.get(inner_ty)?;
                let inner_ty = inner_ty.inner.as_primitive().with_context(|| {
                    format!("Expected primitive type, got {:?}", inner_ty.inner)
                })?;

                let arr = value
                    .as_array()
                    .with_context(|| format!("Expected array, got {:?}", value))?;
                if arr.len() > 1 {
                    anyhow::bail!("Expected array of length 0 or 1, got {:?}", value);
                }

                if arr.is_empty() {
                    Self::Option(None)
                } else {
                    Self::Option(Some(ScalarValue::from_toml(&arr[0], inner_ty)?))
                }
            }
            TypeInner::Enum(e) => {
                let variant = value.as_str().with_context(|| {
                    format!("Expected string for enum variant, got {:?}", value)
                })?;

                let variant = e
                    .members
                    .iter()
                    .find(|(name, _description)| name.as_str() == variant)
                    .with_context(|| {
                        format!(
                            "Expected enum variant to be one of {:?}, got {:?}",
                            e.members, variant
                        )
                    })?;

                Self::Enum(ty_id, variant.0.clone())
            }
        })
    }
}
