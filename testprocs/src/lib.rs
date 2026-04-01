use std::collections::HashMap;

use rs_ts::{ExportShallowType, ExportTypescript, recognize_as};

#[allow(dead_code)]
#[derive(ExportTypescript)]
pub struct User {
    pub name: String,
    pub age: i32,
    pub people: Vec<Vec<Vec<Vec<Vec<HashMap<String, i32>>>>>>,
    pub phone: Option<String>,
}

#[allow(dead_code)]
#[derive(ExportTypescript)]
pub enum Roles {
    User,
    Admin,
    SuperAdmin,
}

pub struct SomeComplexType {}

#[allow(dead_code)]
#[derive(ExportShallowType)]
pub struct OtherType(String);

#[allow(dead_code)]
#[derive(ExportTypescript)]
pub struct SuperUser {
    #[recognize_as("Shallow")]
    pub name: SomeComplexType,
    pub age: i32,
    pub roles: Roles,
    pub meta: Vec<String>,
    pub data: Vec<OtherType>,
}

#[allow(dead_code)]
#[derive(ExportShallowType)]
pub struct Shallow(String);

#[cfg(test)]
mod tests {
    use std::fs;

    fn do_comp(lhs: &str, rhs: &str) -> bool {
        let cleaned = |s: &str| {
            s.trim()
                .replace(" ", "")
                .replace("\n", "")
                .replace("\r", "")
                .replace("\t", "")
                .replace("\r\n", "")
                .trim()
                .to_lowercase()
        };

        let lhs = cleaned(lhs);
        let rhs = cleaned(rhs);

        // ! lengths do not match, simple comp
        if lhs.len() != rhs.len() {
            return false;
        }

        for (lh_ch, rh_ch) in lhs.chars().zip(rhs.chars()) {
            if lh_ch != rh_ch {
                return false;
            }
        }

        true
    }

    //reads a file from the path and asserts when not ok
    fn read_file(path: &str) -> String {
        let fs_result = fs::read_to_string(path);
        assert!(fs_result.is_ok(), "file stream read result failed");
        fs_result.unwrap()
    }

    #[test]
    #[allow(dead_code)]
    fn check_shallow_output() {
        let rh_ts_code = read_file("./types/Shallow.ts");

        let lh_ts_code = r#"
type Shallow = string;

export default Shallow;
        "#;

        assert!(
            do_comp(&rh_ts_code, lh_ts_code),
            "user did not match comparison"
        );
    }

    #[test]
    fn check_user_output() {
        let rh_ts_code = read_file("./types/User.ts");

        let lh_ts_code = r#"
import Nullable from './Nullable';

interface User { 
	name: string;
	age: number;
	people: Array<Array<Array<Array<Array<Record<string, number>>>>>>;
	phone: Nullable<String>;
}

export default User
"#;

        assert!(
            do_comp(&rh_ts_code, lh_ts_code),
            "user did not match comparison"
        );
    }

    #[test]
    fn check_super_user_output() {
        let rh_ts_code = read_file("./types/SuperUser.ts");

        let lh_ts_code = r#"import Shallow from './Shallow';
import Roles from './Roles';
import OtherType from './OtherType';

interface SuperUser {
	name: Shallow;
	age: number;
	roles: Roles;
	meta: Array<string>;
	data: Array<OtherType>;
}

export default SuperUser"#;

        assert!(
            do_comp(&rh_ts_code, lh_ts_code),
            "SUPER user did not match comparison"
        );
    }
}
