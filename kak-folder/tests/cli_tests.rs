use assert_cmd::Command;
use paste::paste;

macro_rules! def_region {
    ($name: ident, $value:literal) => {
        paste! {
            const [<REG_ $name>]: &str = $value;
            const [<FOLD_ $name>]: &str = concat!($value, "|...");
        }
    };
}
const TIMESTAMP_A: &str = "1234";

def_region!(A, "10.10,20.20");
def_region!(OA, "10.15,20.1");

def_region!(B, "30.30,40.40");
def_region!(OB, "30.40,30.50");

macro_rules! test_gen {
    {
				test: $test_name:ident,
				command: $command:literal,
        folds: $folds:literal,
        regions: $regions:literal,
        result: $result:literal,
    } => {
        #[test]
        fn $test_name() {
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_kak-folder"));

            let folds = format!($folds);
            let regions= format!($regions);
            let result = format!($result);

            cmd.arg($command)
                .arg(folds)
                .arg(regions)
                .assert()
                .success()
                .stdout(predicates::str::diff(result));
        }
    };
}

test_gen! {
    test: gen_add_1,
    command: "add",
    folds: "{TIMESTAMP_A} {FOLD_A}",
    regions: "{REG_A}",
    result: "{TIMESTAMP_A} {FOLD_A}\n",
}
test_gen! {
    test: gen_add_2,
    command: "add",
    folds: "{TIMESTAMP_A} {FOLD_A}",
    regions: "{REG_OA}",
    result: "{TIMESTAMP_A} {FOLD_A} {FOLD_OA}\n",
}
test_gen! {
    test: gen_add_3,
    command: "add",
    folds: "{TIMESTAMP_A} {FOLD_A}",
    regions: "{REG_B}",
    result: "{TIMESTAMP_A} {FOLD_A} {FOLD_B}\n",
}

test_gen! {
    test: gen_sub_1,
    command: "remove",
    folds: "{TIMESTAMP_A} {FOLD_A}",
    regions: "{REG_OA}",
    result: "{TIMESTAMP_A}\n",
}
test_gen! {
    test: gen_sub_2,
    command: "remove",
    folds: "{TIMESTAMP_A} {FOLD_A}",
    regions: "{REG_B}",
    result: "{TIMESTAMP_A} {FOLD_A}\n",
}
test_gen! {
    test: gen_sub_3,
    command: "remove",
    folds: "{TIMESTAMP_A} {FOLD_A} {FOLD_B}",
    regions: "{REG_OB}",
    result: "{TIMESTAMP_A} {FOLD_A}\n",
}


#[test]
fn add_1() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kak-folder"));
    cmd.arg("add")
        .arg("1234")
        .arg("10.1,20.1")
        .assert()
        .success()
        .stdout(predicates::str::diff("1234 10.1,20.1|...\n"));
}
#[test]
fn add_2() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kak-folder"));
    cmd.arg("add")
        .env("RUST_BACKTRACE", "1")
        .arg("1234 15.1,15.10|... 20.1,20.10|...")
        .arg("10.10,30.10")
        .assert()
        .success()
        .stdout(predicates::str::diff(
            "1234 15.1,15.10|... 20.1,20.10|... 10.10,30.10|...\n",
        ));
}

#[test]
fn add_3() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kak-folder"));
    cmd.arg("add")
        .env("RUST_BACKTRACE", "1")
        .arg("1234 15.1,15.50|... 20.1,20.10|...")
        .arg("20.1,20.10")
        .assert()
        .success()
        .stdout(predicates::str::diff(
            "1234 15.1,15.50|... 20.1,20.10|...\n",
        ));
}
#[test]
fn remove_1() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kak-folder"));
    cmd.arg("remove")
        .env("RUST_BACKTRACE", "1")
        .arg("1234 15.1,50.1|... 20.1,20.10|...")
        .arg("20.3,20.5")
        .assert()
        .success()
        .stdout(predicates::str::diff("1234\n"));
}
#[test]
fn remove_2() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kak-folder"));
    cmd.arg("remove")
        .env("RUST_BACKTRACE", "1")
        .arg("1234 15.1,50.1|... 20.1,20.10|...")
        .arg("30.3,55.2")
        .assert()
        .success()
        .stdout(predicates::str::diff("1234 20.1,20.10|...\n"));
}

#[test]
fn remove_3() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kak-folder"));
    cmd.arg("remove")
        .env("RUST_BACKTRACE", "1")
        .arg("1646 35.3,35.101|... 35.89,35.52")
        .arg("35.89,35.52")
        .assert()
        .success()
        .stdout(predicates::str::diff("1646\n"));
}
