# RS-TS

RS-TS or "Rust to Typescript" is a proc macro library that is able to convert structs and enums to Typescript files.

It does this by generating a `types` folder and exports each exported type as `name.ts` into the folder.

## Usage

```rust

use rs_ts::{ExportShallowType, ExportTypescript, recognize_as};

#[derive(ExportTypescript)]
pub enum Roles {
    User,
    Admin,
    SuperAdmin,
}

#[derive(ExportShallowType)]
pub struct Name(String);

// Assume this type cannot be exported as a TypeScript object.
pub struct ComplexType {}

// Assume this type cannot be exported as a TypeScript object.
pub struct OtherComplexType {}

#[derive(ExportTypescript)]
pub struct SuperUser {
    #[recognize_as("String")]
    pub name: ComplexType,
    #[recognize_as("Name")]
    pub other_data: OtherComplexType,
    pub age: i32,
    pub roles: Roles,
    pub meta: Vec<String>,
}
```
 
### Output 

`./types/Roles.ts`

```ts

enum Roles {
	User = "User",
	Admin = "Admin",
	SuperAdmin = "SuperAdmin"
}

export default Roles
```

`./types/SuperUser.ts`

```ts
import Name from './Name';
import Roles from './Roles';

interface SuperUser {
    name: string;
    other_data: Name;
    age: number;
    roles: Roles;
    meta: Array<string>;
}

export default SuperUser
```

`./types/Name.ts`

```ts
type Name = string;

export default Name;
```

