use crate::RenameExt;
use crate::{
    language::{Language, SupportedLanguage},
    parser::ParsedData,
    rust_types::{
        RustConst, RustConstExpr, RustEnum, RustEnumVariant, RustField, RustStruct,
        RustTypeAlias, RustTypeFormatError, SpecialRustType,
    },
};
use std::{
    collections::HashMap,
    io::{self, Write},
};

use super::ScopedCrateTypes;

// ReasonML keywords
const REASONML_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "begin", "class", "constraint", "do", "done", "downto",
    "else", "end", "exception", "external", "false", "for", "fun", "function",
    "functor", "if", "in", "include", "inherit", "initializer", "lazy", "let",
    "match", "method", "module", "mutable", "new", "nonrec", "object", "of",
    "open", "or", "private", "rec", "sig", "struct", "switch", "then", "to",
    "true", "try", "type", "val", "virtual", "when", "while", "with",
];

/// All information needed to generate ReasonML type-code
#[derive(Default)]
pub struct ReasonML {
    /// Mappings from Rust type names to ReasonML type names
    pub type_mappings: HashMap<String, String>,
    /// Whether or not to exclude the version header that normally appears at the top of generated code.
    /// If you aren't generating a snapshot test, this setting can just be left as a default (false)
    pub no_version_header: bool,
}

impl Language for ReasonML {
    fn type_map(&mut self) -> &HashMap<String, String> {
        &self.type_mappings
    }
    
    #[allow(clippy::ptr_arg)]
    fn format_simple_type(
        &mut self,
        base: &String,
        _generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        Ok(if let Some(mapped) = self.type_map().get(base) {
            mapped.into()
        } else {
            // For ReasonML, ensure type references are in camelCase
            base.to_camel_case()
        })
    }

    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        if let Some(mapped) = self.type_mappings.get(&special_ty.to_string()) {
            return Ok(mapped.to_owned());
        }
        match special_ty {
            SpecialRustType::Vec(rtype) => {
                Ok(format!("array({0})", self.format_type(rtype, generic_types)?))
            }
            SpecialRustType::Array(rtype, _len) => {
                let formatted_type = self.format_type(rtype, generic_types)?;
                Ok(format!("array({0})", formatted_type))
            }
            SpecialRustType::Slice(rtype) => {
                Ok(format!("array({0})", self.format_type(rtype, generic_types)?))
            }
            SpecialRustType::Option(rtype) => {
                Ok(format!("option({0})", self.format_type(rtype, generic_types)?))
            }
            SpecialRustType::HashMap(_rtype1, rtype2) => Ok(format!(
                "Js.Dict.t({0})",
                self.format_type(rtype2, generic_types)?
            )),
            SpecialRustType::Unit => Ok("unit".into()),
            SpecialRustType::DateTime => Ok("Js.Date.t".into()),
            SpecialRustType::String => Ok("string".into()),
            SpecialRustType::Char => Ok("string".into()),
            SpecialRustType::I8
            | SpecialRustType::U8
            | SpecialRustType::I16
            | SpecialRustType::U16
            | SpecialRustType::I32
            | SpecialRustType::U32
            | SpecialRustType::I54
            | SpecialRustType::U53
            | SpecialRustType::F32
            | SpecialRustType::F64 => Ok("float".into()),
            SpecialRustType::Bool => Ok("bool".into()),
            SpecialRustType::U64
            | SpecialRustType::I64
            | SpecialRustType::ISize
            | SpecialRustType::USize => {
                panic!("64 bit types not allowed in Typeshare")
            }
        }
    }

    fn begin_file(&mut self, w: &mut dyn Write, _parsed_data: &ParsedData) -> io::Result<()> {
        if !self.no_version_header {
            writeln!(w, "/*")?;
            writeln!(w, " * Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
            writeln!(w, " */")?;
            writeln!(w)?;
        }
        Ok(())
    }

    fn write_type_alias(&mut self, w: &mut dyn Write, ty: &RustTypeAlias) -> io::Result<()> {
        self.write_comments(w, 0, &ty.comments)?;

        let r#type = self
            .format_type(&ty.r#type, ty.generic_types.as_slice())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let generic_params = if !ty.generic_types.is_empty() {
            format!("('{})", ty.generic_types.join(", '"))
        } else {
            String::new()
        };

        writeln!(
            w,
            "type {}{} = {};\n",
            ty.id.renamed.to_camel_case(),
            generic_params,
            r#type,
        )?;

        Ok(())
    }

    fn write_const(&mut self, w: &mut dyn Write, c: &RustConst) -> io::Result<()> {
        match c.expr {
            RustConstExpr::Int(val) => {
                let const_type = self
                    .format_type(&c.r#type, &[])
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                writeln!(
                    w,
                    "let {} = ({}: {});",
                    c.id.renamed.to_snake_case().to_uppercase(),
                    val,
                    const_type
                )
            }
        }
    }

    fn write_struct(&mut self, w: &mut dyn Write, rs: &RustStruct) -> io::Result<()> {
        self.write_comments(w, 0, &rs.comments)?;

        let generic_params = if !rs.generic_types.is_empty() {
            format!("('{})", rs.generic_types.join(", '"))
        } else {
            String::new()
        };

        let type_name = rs.id.renamed.to_camel_case();
        
        // Handle empty structs as opaque types
        if rs.fields.is_empty() {
            return writeln!(w, "type {};", type_name);
        }
        
        writeln!(
            w,
            "type {}{} = {{",
            type_name,
            generic_params
        )?;

        rs.fields
            .iter()
            .try_for_each(|f| self.write_field(w, f, rs.generic_types.as_slice()))?;

        writeln!(w, "}};\n")
    }

    fn write_enum(&mut self, w: &mut dyn Write, e: &RustEnum) -> io::Result<()> {
        self.write_comments(w, 0, &e.shared().comments)?;

        let generic_params = if !e.shared().generic_types.is_empty() {
            format!("('{})", e.shared().generic_types.join(", '"))
        } else {
            String::new()
        };

        match e {
            RustEnum::Unit(shared) => {
                writeln!(
                    w,
                    "type {}{} =",
                    shared.id.renamed.to_camel_case(),
                    generic_params
                )?;

                self.write_enum_variants(w, e)?;

                writeln!(w, ";\n")
            }
            RustEnum::Algebraic { shared, .. } => {
                // ReasonML doesn't support serde(tag, content, or rename) style enums
                // Replace the enum comment with our unsupported message
                writeln!(w, "/* Unsupported Serde Serialisation */")?;
                writeln!(w, "type {};\n", shared.id.renamed.to_camel_case())
            }
        }
    }

    fn write_imports(
        &mut self,
        w: &mut dyn Write,
        imports: ScopedCrateTypes<'_>,
    ) -> std::io::Result<()> {
        for (path, _) in imports {
            writeln!(w, "open {};\n", path)?;
        }
        Ok(())
    }

    fn ignored_reference_types(&self) -> Vec<&str> {
        self.type_mappings.keys().map(|s| s.as_str()).collect()
    }
}

impl ReasonML {
    fn write_enum_variants(&mut self, w: &mut dyn Write, e: &RustEnum) -> io::Result<()> {
        match e {
            RustEnum::Unit(shared) => {
                let variants = &shared.variants;
                for v in variants.iter() {
                    match v {
                        RustEnumVariant::Unit(shared) => {
                            self.write_comments(w, 1, &shared.comments)?;
                            writeln!(w, "  | {}", shared.id.renamed)?;
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(())
            }
            RustEnum::Algebraic {
                tag_key,
                content_key,
                shared,
            } => {
                let variants = &shared.variants;
                for variant in variants.iter() {
                    match variant {
                        RustEnumVariant::Unit(shared) => {
                            self.write_comments(w, 1, &shared.comments)?;
                            writeln!(
                                w,
                                "  | {}({}: string)", 
                                shared.id.renamed,
                                tag_key
                            )?;
                        }
                        RustEnumVariant::Tuple { ty, shared } => {
                            self.write_comments(w, 1, &shared.comments)?;
                            let r#type = self
                                .format_type(ty, e.shared().generic_types.as_slice())
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                            writeln!(
                                w,
                                "  | {}({}: string, {}: {})", 
                                shared.id.renamed,
                                tag_key,
                                content_key,
                                r#type
                            )?;
                        }
                        RustEnumVariant::AnonymousStruct { fields, shared } => {
                            self.write_comments(w, 1, &shared.comments)?;
                            writeln!(
                                w,
                                "  | {}({}: string, {}: {{",
                                shared.id.renamed, tag_key, content_key
                            )?;
                            
                            for field in fields {
                                self.write_field(w, field, e.shared().generic_types.as_slice())?;
                            }
                            
                            writeln!(w, "  }})")?;
                        }
                    }
                }
                Ok(())
            }
        }
    }

    fn write_field(
        &mut self,
        w: &mut dyn Write,
        field: &RustField,
        generic_types: &[String],
    ) -> io::Result<()> {
        self.write_comments(w, 1, &field.comments)?;
        let reasonml_ty: String = match field.type_override(SupportedLanguage::TypeScript) {
            Some(type_override) => type_override.to_owned(),
            None => self
                .format_type(&field.ty, generic_types)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
        };
        
        // If the type itself is already optional (from Option<T>), don't double-wrap it
        let type_str = reasonml_ty;
        
        writeln!(
            w,
            "    {}: {},",
            reasonml_property_aware_rename(&field.id.renamed),
            type_str
        )?;

        Ok(())
    }

    fn write_comments(
        &mut self,
        w: &mut dyn Write,
        indent: usize,
        comments: &[String],
    ) -> io::Result<()> {
        // Only attempt to write a comment if there are some, otherwise we're Ok()
        if !comments.is_empty() {
            let comment: String = {
                let tab_indent = "  ".repeat(indent);
                // If there's only one comment then keep it on the same line, otherwise we'll make a nice multi-line comment
                if comments.len() == 1 {
                    format!("{}/* {} */", tab_indent, comments.first().unwrap())
                } else {
                    let joined_comments = comments.join(&format!("\n{} * ", tab_indent));
                    format!(
                        "{tab}/*
{tab} * {comment}
{tab} */",
                        tab = tab_indent,
                        comment = joined_comments
                    )
                }
            };
            writeln!(w, "{}", comment)?;
        }
        Ok(())
    }
}

fn reasonml_property_aware_rename(name: &str) -> String {
    // Check if the name contains hyphens or is a keyword
    if name.contains('-') || REASONML_KEYWORDS.contains(&name) {
        return format!("\"{0}\"", name);
    }
    name.to_string()
}

