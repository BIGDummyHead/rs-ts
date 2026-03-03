# Typescripted

A proc macro library that is able to convert structs and enums to Typescript files.

Generates a `types` folder and exports each exported type as `name.ts` into the types folder.

## Usage

```rust
 #[derive(ExportTypescript)]
 struct User {
     pub uid: i32;
     pub display_name: string;
     pub role: Role;
 }

 #[derive(ExportTypescript)]
 enum Role {
     User,
     Admin
 }
```
