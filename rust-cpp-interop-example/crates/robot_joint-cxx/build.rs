fn main() {
    cxx_build::bridge("src/lib.rs")
        .std("c++20")
        .flag_if_supported("-std=c++20")
        .compile("robot_joint_cxx");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/robot_joint/robot_joint.hpp");
}
