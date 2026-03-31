mod aux_funcs;
mod import_data;
mod type_kit;

use import_data::*;
use type_kit::*;

use std::collections::HashMap;

use crate::aux_funcs::{write_to_file, write_type_to_file};
use proc_macro::TokenStream;
use quote::quote;
use std::sync::{Arc, LazyLock};
use syn::{Data, DeriveInput, parse_macro_input};

type TypeConverter = HashMap<&'static str, Arc<TypeKit>>;

static TYPE_CONVERTER: LazyLock<TypeConverter> = LazyLock::new(|| {
    // Converter for Rust basic types to TypeScript types.
    // Custom types can be still be used as they rely on other interfaces.

    //default import types
    let number = Arc::new(TypeKit::no_import("number"));
    let bool_kit = Arc::new(TypeKit::no_import("boolean"));
    let string = Arc::new(TypeKit::no_import("string"));
    let void = Arc::new(TypeKit::no_import("void"));
    let record = Arc::new(TypeKit::no_import("Record"));
    let object = Arc::new(TypeKit::no_import("Object"));
    let any = Arc::new(TypeKit::no_import("any"));
    let array = Arc::new(TypeKit::no_import("Array"));

    //import types
    // import {type_name: Nullable} from "./{file_name: Nullable}";

    let nullable = Arc::new(TypeKit::import(
        "Nullable",
        ImportData {
            type_name: String::from("Nullable"),
            file_name: String::from("Nullable"),
        },
    ));

    let mut type_converter: TypeConverter = HashMap::new();
    type_converter.insert("i8", number.clone());
    type_converter.insert("i16", number.clone());
    type_converter.insert("i32", number.clone());
    type_converter.insert("i64", number.clone());
    type_converter.insert("i128", number.clone());
    type_converter.insert("isize", number.clone());
    type_converter.insert("u8", number.clone());
    type_converter.insert("u16", number.clone());
    type_converter.insert("u32", number.clone());
    type_converter.insert("u64", number.clone());
    type_converter.insert("u128", number.clone());
    type_converter.insert("usize", number.clone());
    type_converter.insert("f32", number.clone());
    type_converter.insert("f64", number.clone());
    type_converter.insert("bool", bool_kit.clone());
    type_converter.insert("String", string.clone());
    type_converter.insert("str", string.clone());
    type_converter.insert("()", void.clone());
    type_converter.insert("Result", any.clone());
    type_converter.insert("HashMap", record.clone());
    type_converter.insert("BTreeMap", object.clone());
    type_converter.insert("Vec", array.clone());
    type_converter.insert("Option", nullable.clone());

    type_converter
});

static TYPES_FOLDER: LazyLock<&'static str> = LazyLock::new(|| "./types");

/// # Take Generic, takes a whole type such as Vec<Vec<String>> and returns the type and the inner generic such as:
///
/// (Vec, Vec<String>) this process can be repeated until an option none is returned back as the second argument.
fn take_generic(whole_type: &str) -> (String, Option<String>) {
    //replace each space in the type to be compact
    // Vec < String > -> Vec<String>
    let whole_type = whole_type.replace(" ", "");

    let chars = whole_type.chars();

    let mut inner_type = String::new();

    //the owner type
    let mut owner_type = String::new();

    let mut generics_encountered = 0;
    for ch in chars {
        // Vec< HashMap< String, Vec< String > > >
        //   -^       -^           -^
        if ch == '<' {
            // add the '<' IF we have encountered another '<' before.
            if generics_encountered > 0 {
                inner_type.push(ch);
            }

            generics_encountered += 1;
            continue;
        }

        // we can ignore characters for example:
        // Vec< HashMap< String, Vec< String > > >
        //-^^^^                               -^-^
        if generics_encountered == 0 {
            owner_type.push(ch); //add this character to the owner...
            continue;
        }

        //indicate that the wrapper has ended
        if ch == '>' {
            generics_encountered -= 1;

            if generics_encountered == 0 {
                break;
            }
        }

        //add char
        inner_type.push(ch);
    }

    if inner_type.is_empty() {
        (owner_type, None)
    } else {
        (owner_type, Some(inner_type))
    }
}

//cleans the inner generic (final), maps to a formal type conversion
fn clean_final_inner(type_converter: &TypeConverter, inner: &str) -> String {
    let fallback_type = |type_name: &str| {
        type_converter
            .get(type_name)
            .map(|tk| tk.name.to_string())
            .unwrap_or(type_name.to_string())
    };

    //there is nothing to convert it is a single type
    if !inner.contains(",") {
        return fallback_type(inner);
    }

    let inner = inner.to_string();

    //the current type encountered...
    let mut current_ty = String::new();

    //our final output
    let mut output = String::new();

    for ch in inner.chars() {
        //push (...)
        if ch == '(' || ch == ')' {
            continue;
        }

        //type is done
        if ch == ',' {
            let ty = fallback_type(current_ty.as_str());

            //reset the string
            current_ty = String::new();

            //converted type
            output.push_str(&ty);
            //add comma and space
            output.push(ch);
            output.push(' ');

            continue;
        }

        //push the ch into the current type
        current_ty.push(ch);
    }

    let current_ty = fallback_type(&current_ty);

    output.push_str(&current_ty);

    output
}

/// # Get Formal Type Name
///
/// From the type's unformal name (known in Rust) converts to a TypeScript name.
fn get_formal_type_name(full_type_name: &str, imports: &mut HashMap<String, String>) -> String {
    if full_type_name.contains("<") {
        let mut created_name = String::new();

        let mut ty = full_type_name.to_string();

        let mut additional_enders = 0;

        while let (owner, Some(inner)) = take_generic(&ty) {
            let type_kit = TYPE_CONVERTER.get(owner.as_str());

            let owner: String = type_kit.map(|t| t.name.to_string()).unwrap_or(owner);

            if let Some(kit) = type_kit {
                if let Some(kit_import) = &kit.import {
                    imports.insert(kit_import.type_name.clone(), kit_import.as_string());
                }
            }

            //add the owner name (converted) to the whole name
            created_name.push_str(&owner);
            created_name.push_str("<");

            //the inner given is also generic
            if inner.contains("<") {
                ty = inner;
                additional_enders += 1;
                continue;
            } else {
                let inner = clean_final_inner(&TYPE_CONVERTER, &inner);
                //inner needs to be cleaned
                created_name.push_str(inner.trim());
                created_name.push_str(">");
                break;
            }
        }

        for _ in 0..additional_enders {
            created_name.push('>');
        }

        created_name
    } else {
        // ? handle non generics
        //convert our unsanitized Rust type to TypeScript.
        let non_generic_name = if let Some(type_kit) = TYPE_CONVERTER.get(full_type_name) {
            if let Some(import_data) = &type_kit.import {
                imports.insert(full_type_name.to_string(), import_data.as_string());
            }

            type_kit.name.to_string()
        } else {
            //ty Vec < String >
            //import { X } from './X';
            let import_value = ImportData {
                type_name: full_type_name.to_string(),
                file_name: full_type_name.to_string(),
            };

            imports.insert(full_type_name.to_string(), import_value.as_string());

            full_type_name.to_string()
        };
        non_generic_name
    }
}

/// Formats and pushes appropriate imports
fn push_imports(imports: &HashMap<String, String>, output: &mut String) -> () {
    for (_, import) in imports {
        output.push_str(&format!("{import}\r\n"));
    }
}

/// # Export TypeScript
///
/// Exports the Struct or Enum to a TypeScript (.ts) file.
///
/// The files that are generated are stored in `./types/*.ts`
///
/// The following example generates the directory `types` then *2* files `User.ts` and `Role.ts`.
///
/// It is important to note that when generated it is assumed that Role will ALSO be exported.
///
/// Usage:
///
/// ```rs
/// #[derive(ExportTypescript)]
/// struct User {
///     pub uid: i32,
///     pub display_name: string,
///     pub role: Role,
///     pub meta: Vec<String>
/// }
///
/// #[derive(ExportTypescript)]
/// enum Role {
///     User,
///     Admin
/// }
/// ```
///
/// Generated data:
///
/// User.ts:
///
/// ```ts
/// import Role from './Role.ts';
///
/// interface User {
///     uid: number;
///     display_name: string;
///     role: Role;
///     meta: Array<string>
/// }
///
/// export default User;
///
/// ```
///
/// Role.ts:
///
/// ```ts
/// enum Role {
///     User = "User",
///     Admin = "Admin"
/// }
///
/// export default Role;
/// ```
#[proc_macro_derive(ExportTypescript, attributes(recognize_as))]
pub fn export_typescript(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // a maps each field into a Vec<String> containing their field names.
    let mut field_data_string: String = match input.data {
        Data::Struct(ds) => {
            //export interface NAME {
            let mut interface_export_output: String = "interface ".to_string();
            interface_export_output.push_str(input.ident.to_string().as_str());
            interface_export_output.push_str(" {\r\n");

            //imports
            let mut imports: HashMap<String, String> = HashMap::new();

            let mut output_types: Vec<String> = Vec::new();
            for field in ds.fields {
                let field_type = quote!(#field.ty).to_string();

                //empty or non-public
                if !field_type.contains("pub") || field_type.is_empty() {
                    continue;
                }

                // [documentXXXX] pub
                let field_type = field_type
                    .trim()
                    .rsplit_once("pub")
                    .map(|(_, r)| {
                        r.split_once(":")
                            .map(|(l, r)| (l.trim(), r.trim()))
                            .unwrap_or(("", ""))
                    })
                    .and_then(|(l, r)| {
                        if l.is_empty() || r.is_empty() {
                            None
                        } else {
                            let (ty_name, _) = r.split_once(".").unwrap();

                            Some((l.trim(), ty_name.trim()))
                        }
                    });

                //nothing to do
                if field_type.is_none() {
                    continue;
                }

                let (name, full_type_name) = field_type.unwrap();
                let mut full_type_name = full_type_name.to_string();

                for attr in &field.attrs {
                    if attr.path().is_ident("recognize_as") {
                        let lit: syn::LitStr = attr.parse_args().unwrap();
                        full_type_name = lit.value();
                    }
                }
                let formal_type_name = get_formal_type_name(&full_type_name, &mut imports);

                output_types.push(format!("\t{name}: {formal_type_name};"));
            }

            let output_types = output_types.join("\r\n");
            interface_export_output.push_str(&output_types);
            interface_export_output.push_str("\r\n}");

            let mut output = String::new();

            push_imports(&imports, &mut output);

            output.push_str("\r\n");
            output.push_str(&interface_export_output);

            output
        }
        Data::Enum(de) => {
            let mut output = String::new();

            let enum_name = input.ident.to_string();
            output.push_str(&format!("enum {enum_name} {{\r\n"));

            //identifiers within the enum
            let fields: Vec<String> = de
                .variants
                .into_iter()
                .map(|f| quote!(#f.ident).to_string())
                .map(|s| {
                    //remove the #doc comments and other macros
                    let name = if let Some((_, doc_removed_name)) = s.rsplit_once("]") {
                        doc_removed_name
                    } else {
                        &s
                    };

                    name.trim().to_string()
                })
                .map(|s| {
                    let spl = s.split_once(".");

                    if let Some((left, _)) = spl {
                        let left = left.trim();
                        format!("\t{left} = \"{left}\"")
                    } else {
                        "".to_string()
                    }
                })
                .filter(|s| !s.is_empty())
                .collect();

            let fields = fields.join(",\r\n");

            output.push_str(&fields);

            output.push_str("\r\n}");

            output
        }
        _ => panic!("this is not yet supported."),
    };

    //convert field data into vec<u8>
    field_data_string.push_str("\r\n\r\nexport default ");
    field_data_string.push_str(&format!("{}", input.ident.to_string()));
    let output_data = field_data_string.into_bytes();

    write_type_to_file(input.ident.to_string(), &output_data);

    let folder = TYPES_FOLDER.trim();

    let default_file_collection = HashMap::from([(
        "Nullable",
        "//[NOTE]: This file was auto-generated for easier conversion of Option<T>\r\ntype Nullable<T> = T | null | undefined;\r\n\r\nexport default Nullable;",
    )]);

    for (file, content) in default_file_collection {
        let full_path = format!("{folder}/{file}.ts");
        write_to_file(&full_path, content.as_bytes());
    }

    //do nothing.  data collected and wrote
    TokenStream::new()
}

/// # Export Shallow Type
///
/// Exports a shallow struct into a type, in TypeScript.
///
/// For example:
///
/// ```
/// use rs_ts::ExportShallowType;
///
/// #[derive(ExportShallowType)]
/// pub struct Name(String);
/// ```
///
/// Generated `Name.ts` in the `types` folder with the following content:
///
/// ```ts
/// type Name = string;
///
/// export default Name;
/// ```
#[proc_macro_derive(ExportShallowType)]
pub fn export_shallow_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(struct_data) = input.data else {
        panic!("this derive macro only works on structs");
    };

    let type_ident = input.ident.to_string();
    let mut fields = struct_data.fields.into_iter();

    if fields.len() > 1 || fields.len() == 0 {
        panic!("only support single field");
    }

    let first = fields
        .next()
        .unwrap_or_else(|| panic!("could not obtain single field"));

    let whole_field_ty = quote!(#first.ty).to_string();

    // the Type name : String
    let mut field_type_name = whole_field_ty
        .split_once(".")
        .and_then(|(type_name, _ext)| Some(type_name.trim()))
        .unwrap_or_else(|| panic!("could not obtain field type"));

    //pub String for example
    if field_type_name.contains("pub") {
        let spl = field_type_name.split_once("pub").and_then(|(l, r)| {
            Some(r.trim())
        }).unwrap_or_else(|| panic!("could not obtain field type"));

        field_type_name = spl;
    }

    //output stream
    let mut output = String::new();

    let mut imports = HashMap::new();
    let field_type_name = get_formal_type_name(field_type_name, &mut imports);

    push_imports(&imports, &mut output);
    output.push_str("\r\ntype ");
    output.push_str(&type_ident);
    output.push_str(" = ");
    output.push_str(&field_type_name);
    output.push_str(";\r\n\r\nexport default ");
    output.push_str(&type_ident);
    output.push_str(";");

    write_type_to_file(type_ident, output.as_bytes());

    TokenStream::new()
}

/// # Recognize As
///
/// Applied to fields of a struct or enum that derives ExportTypeScript.
///
/// Allows you to rename the identified field type to something else.
///
/// For example
///
/// ```
/// use rs_ts::ExportTypescript;
///
/// pub struct ComplexType {}
///
/// #[derive(ExportTypescript)]
/// pub struct User {
///     pub name: String,
///     #[recognize_as("String")]
///     pub meta: ComplexType
/// }
///```
///
/// The 'ComplexType' type will now be identified as a String type when parsed by the `ExportTypeScript` macro. Allowing for easy type conversion.
#[proc_macro_attribute]
pub fn recognize_as(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    TokenStream::new()
}
