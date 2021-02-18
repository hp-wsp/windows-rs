use super::*;

#[derive(Debug)]
pub struct MethodSignature {
    pub params: Vec<MethodParam>,
    pub return_type: Option<Signature>,
}

#[derive(Debug)]
pub struct MethodParam {
    pub param: tables::Param,
    pub signature: Signature,
}

impl MethodSignature {
    pub fn dependencies(&self) -> Vec<tables::TypeDef> {
        self.return_type
            .iter()
            .filter_map(|s| s.definition())
            .chain(self.params.iter().filter_map(|p| p.signature.definition()))
            .collect()
    }
}

// #[derive(Debug)]
// pub struct MethodSignature {
//     pub method: winmd::MethodDef,
//     pub params: Vec<Type>,
//     pub return_type: Option<Type>,
// }

// impl MethodSignature {
//     pub fn new(
//         method: &winmd::MethodDef,
//         generics: &[TypeKind],
//         calling_namespace: &'static str,
//     ) -> Self {
//         let mut params: Vec<winmd::Param> = method.params().collect();

//         let return_param = if !params.is_empty() && params[0].sequence() == 0 {
//             Some(params.remove(0))
//         } else {
//             None
//         };

//         let mut blob = method.blob();
//         blob.read_unsigned(); // First byte of MethodDefSig is not used.
//         let param_count = blob.read_unsigned() as usize;

//         let return_type =
//             Type::from_blob(&mut blob, return_param, generics, calling_namespace, true);

//         debug_assert!(params.len() == param_count);
//         let mut param_types = Vec::with_capacity(param_count);

//         for param in params {
//             param_types.push(
//                 Type::from_blob(&mut blob, Some(param), generics, calling_namespace, false)
//                     .unwrap(),
//             );
//         }

//         Self {
//             method: *method,
//             params: param_types,
//             return_type,
//         }
//     }

//     pub fn dependencies(&self) -> Vec<winmd::TypeDef> {
//         let mut defs = Vec::new();

//         if let Some(t) = &self.return_type {
//             defs.append(&mut t.kind.dependencies());
//         }

//         for param in &self.params {
//             defs.append(&mut param.kind.dependencies());
//         }

//         defs
//     }
// }
