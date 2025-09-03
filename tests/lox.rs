use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn math1() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/math1.lox")
        .assert()
        .success()
        .stdout("Yes\n");
}

#[test]
fn math2() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/math2.lox")
        .assert()
        .success()
        .stdout("Yes\n");
}

#[test]
fn math3() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/math3.lox")
        .assert()
        .success()
        .stdout("Yes\n");
}

#[test]
fn math4() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/math4.lox")
        .assert()
        .success()
        .stdout("Yes\n");
}

#[test]
fn math5() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/math5.lox")
        .assert()
        .success()
        .stdout("Yes\n");
}

#[test]
fn string() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/string.lox")
        .assert()
        .success()
        .stdout("a1b2c3d4\n");
}

#[test]
fn while_test() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/while.lox")
        .assert()
        .success()
        .stdout("55\n");
}

#[test]
fn recursion() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/recursion.lox")
        .assert()
        .success()
        .stdout("8\n");
}

#[test]
fn class() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/class.lox")
        .assert()
        .success()
        .stdout("3\n");
}

#[test]
fn superclass() {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg("tests/input/superclass.lox")
        .assert()
        .success()
        .stdout("22\n");
}
