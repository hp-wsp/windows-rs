use super::*;

// TODO: this replaces TypeKind, TypeName, and TypeDefinition
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum ElementType {
    NotYetSupported,
    Void,
    Bool,
    Char,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    ISize,
    USize,
    String,
    Object,
    Guid,
    IUnknown,
    ErrorCode,
    Bool32,
    Matrix3x2,
    TypeName,
    GenericParam(GenericParam),

    Function(Function),
    Constant(Field),
    Class(Class),
    Interface(Interface),
    ComInterface(ComInterface),
    Enum(Enum),
    Struct(Struct),
    Delegate(Delegate),
    Callback(Callback),
}

impl ElementType {
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0x01 => Some(Self::Void),
            0x02 => Some(Self::Bool),
            0x03 => Some(Self::Char),
            0x04 => Some(Self::I8),
            0x05 => Some(Self::U8),
            0x06 => Some(Self::I16),
            0x07 => Some(Self::U16),
            0x08 => Some(Self::I32),
            0x09 => Some(Self::U32),
            0x0a => Some(Self::I64),
            0x0b => Some(Self::U64),
            0x0c => Some(Self::F32),
            0x0d => Some(Self::F64),
            0x18 => Some(Self::ISize),
            0x19 => Some(Self::USize),
            0x0e => Some(Self::String),
            _ => None,
        }
    }

    pub fn from_blob(blob: &mut Blob, generics: &[Self]) -> Self {
        let code = blob.read_unsigned();

        if let Some(code) = Self::from_code(code) {
            return code;
        }

        match code {
            0x11 | 0x12 => {
                let code = TypeDefOrRef::decode(blob.reader, blob.read_unsigned(), blob.file_index);

                match code {
                    TypeDefOrRef::TypeRef(type_ref) => match type_ref.full_name() {
                        ("System", "Guid") | ("Windows.Win32.Com", "Guid") => Self::Guid,
                        ("Windows.Win32.Com", "IUnknown") => Self::IUnknown,
                        ("Windows.Foundation", "HResult") => Self::ErrorCode,
                        ("Windows.Win32.Com", "HRESULT") => Self::ErrorCode,
                        ("Windows.Win32.SystemServices", "BOOL") => Self::Bool32,
                        ("Windows.Win32.SystemServices", "LARGE_INTEGER") => Self::I64,
                        ("Windows.Win32.SystemServices", "ULARGE_INTEGER") => Self::U64,
                        ("Windows.Win32.Direct2D", "D2D_MATRIX_3X2_F") => Self::Matrix3x2,
                        ("System", "Type") => Self::TypeName,
                        ("", _) => Self::NotYetSupported,
                        _ => Self::from_type_def(type_ref.resolve(), Vec::new()),
                    },
                    TypeDefOrRef::TypeDef(type_def) => { // TODO: does this ever happen?
                        Self::from_type_def(type_def, Vec::new())
                    }
                    _ => unexpected!(),
                }
            }
            0x13 => generics[blob.read_unsigned() as usize].clone(),
            0x14 => Self::NotYetSupported, // arrays
            0x15 => {
                let def = GenericType::from_blob(blob, generics);
                match def.def.category() {
                    TypeCategory::Interface => Self::Interface(Interface(def)),
                    TypeCategory::Delegate => Self::Delegate(Delegate(def)),
                    _ => unexpected!(),
                }
            }
            _ => unexpected!(),
        }
    }

    pub fn from_type_def(def: TypeDef, generics: Vec<Self>) -> Self {
        match def.category() {
            TypeCategory::Interface => {
                if def.is_winrt() {
                    Self::Interface(Interface(GenericType::from_type_def(def, generics)))
                } else {
                    Self::ComInterface(ComInterface(GenericType::from_type_def(def, generics)))
                }
            }
            TypeCategory::Class => Self::Class(Class(GenericType::from_type_def(def, generics))),
            TypeCategory::Enum => Self::Enum(Enum(def)),
            TypeCategory::Struct => Self::Struct(Struct(def)),
            TypeCategory::Delegate => {
                if def.is_winrt() {
                    Self::Delegate(Delegate(GenericType::from_type_def(def, generics)))
                } else {
                    Self::Callback(Callback(def))
                }
            }
            _ => unexpected!(),
        }
    }

    pub fn gen_name(&self, gen: Gen) -> TokenStream {
        match self {
            Self::Void => quote! { ::std::ffi::c_void },
            Self::Bool => quote! { bool },
            Self::Char => quote! { u16 },
            Self::I8 => quote! { i8 },
            Self::U8 => quote! { u8 },
            Self::I16 => quote! { i16 },
            Self::U16 => quote! { u16 },
            Self::I32 => quote! { i32 },
            Self::U32 => quote! { u32 },
            Self::I64 => quote! { i64 },
            Self::U64 => quote! { u64 },
            Self::F32 => quote! { f32 },
            Self::F64 => quote! { f64 },
            Self::ISize => quote! { isize },
            Self::USize => quote! { usize },
            Self::String => {
                let windows = gen.windows();
                quote! { #windows HString }
            }
            Self::Object => {
                let windows = gen.windows();
                quote! { #windows Object }
            }
            Self::Guid => {
                let windows = gen.windows();
                quote! { #windows Guid }
            }
            Self::IUnknown => {
                let windows = gen.windows();
                quote! { #windows IUnknown }
            }
            Self::ErrorCode => {
                let windows = gen.windows();
                quote! { #windows ErrorCode }
            }
            Self::Bool32 => {
                let windows = gen.windows();
                quote! { #windows BOOL }
            }
            Self::Matrix3x2 => {
                let windows = gen.windows();
                quote! { #windows foundation::numerics::Matrix3x2 }
            }
            Self::NotYetSupported => {
                let windows = gen.windows();
                quote! { #windows NOT_YET_SUPPORTED_TYPE }
            }
            Self::GenericParam(generic) => generic.gen_name(),
            Self::Function(t) => t.gen_name(),
            Self::Constant(t) => t.gen_name(),
            Self::Class(t) => t.0.gen_name(gen),
            Self::Interface(t) => t.0.gen_name(gen),
            Self::ComInterface(t) => t.0.gen_name(gen),
            Self::Enum(t) => t.0.gen_name(gen),
            Self::Struct(t) => t.0.gen_name(gen),
            Self::Delegate(t) => t.0.gen_name(gen),
            Self::Callback(t) => t.0.gen_name(gen),
            _ => unexpected!(),
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            Self::Object | Self::IUnknown | Self::Function(_) | Self::Interface(_) | Self::Class(_) | Self::ComInterface(_) | Self::Delegate(_) | Self::Callback(_) => true,
            _ => false,
        }
    }

    pub fn signature(&self) -> String {
        match self {
            Self::Bool => "b1".to_owned(),
            Self::Char => "c2".to_owned(),
            Self::I8 => "i1".to_owned(),
            Self::U8 => "u1".to_owned(),
            Self::I16 => "i2".to_owned(),
            Self::U16 => "u2".to_owned(),
            Self::I32 => "i4".to_owned(),
            Self::U32 => "u4".to_owned(),
            Self::I64 => "i8".to_owned(),
            Self::U64 => "u8".to_owned(),
            Self::F32 => "f4".to_owned(),
            Self::F64 => "f8".to_owned(),
            Self::String => "string".to_owned(),
            Self::Object => "cinterface(IInspectable)".to_owned(),
            Self::Guid => "g16".to_owned(),
            Self::Class(t) => t.signature(),
            Self::Interface(t) => t.signature(),
            Self::Enum(t) => t.signature(),
            Self::Struct(t) => t.signature(),
            Self::Delegate(t) => t.signature(),
            _ => unexpected!(),
        }
    }

    pub fn dependencies(&self) -> Vec<TypeDef> {
        match self {
            Self::Function(t) => t.dependencies(),
            Self::Constant(t) => t.dependencies(),
            Self::Class(t) => t.dependencies(),
            Self::Interface(t) => t.dependencies(),
            Self::ComInterface(t) => t.dependencies(),
            Self::Struct(t) => t.dependencies(),
            Self::Delegate(t) => t.dependencies(),
            Self::Callback(t) => t.dependencies(),
            _ => Vec::new(),
        }
    }

    pub fn gen(&self, gen: Gen) -> TokenStream {
        match self {
            Self::Function(t) => t.gen(gen),
            Self::Constant(t) => t.gen(gen),
            Self::Class(t) => t.gen(gen),
            Self::Interface(t) => t.gen(gen),
            Self::ComInterface(t) => t.gen(gen),
            Self::Enum(t) => t.gen(gen),
            Self::Struct(t) => t.gen(gen),
            Self::Delegate(t) => t.gen(gen),
            Self::Callback(t) => t.gen(gen),
            _ => unexpected!(),
        }
    }

    #[cfg(test)]
    pub fn as_struct(&self) -> Struct {
        if let Self::Struct(value) = self { value.clone() } else { unexpected!(); }
    }

    #[cfg(test)]
    pub fn as_interface(&self) -> Interface {
        if let Self::Interface(value) = self { value.clone() } else { unexpected!(); }
    }

    #[cfg(test)]
    pub fn as_class(&self) -> Class {
        if let Self::Class(value) = self { value.clone() } else { unexpected!(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        assert_eq!(ElementType::Bool.gen_name(Gen::Absolute).as_str(), "bool");
    }
}