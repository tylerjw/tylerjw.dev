#!/usr/bin/env python3
import elsie
from my_layouts import *
from elsie.ext import unordered_list


def my_page_numbering(slides: elsie.SlideDeck):
    total = len(slides)
    for i, slide in enumerate(slides):
        slide.box(x="[100%]", y="[100%]", p_right=30, p_bottom=30).text(
            f"{i+1} ~grayed_text{{of {total}}}", elsie.TextStyle(align="right", size=36)
        )


slides = init_deck(font="Noto Sans")


@slides.slide(debug_boxes=False)
def title(slide):
    content = logo_header_slide(slide, "")
    content.box(width="fill").text(
        "Rust/C++ Interop", elsie.TextStyle(size=160, bold=True)
    )
    content.box(width="fill").text(
        "Boulder Rust Meetup", elsie.TextStyle(size=36, bold=True, italic=True)
    )
    content.box(width="fill", p_top=10).text("February 7, 2024")
    content.box(width="fill", p_top=180).text("Tyler Weaver")
    content.box(width="fill").text("Staff Software Engineer\nmaybe@tylerjw.dev")


@slides.slide(debug_boxes=False)
def author(slide):
    text_area = text_slide(slide, "Tyler Weaver")
    lst = unordered_list(text_area)
    lst.item().text("Regular C++ Programmer")
    lst.item().text("Rust Cult Member")
    lst.item().text("Open-source Robotcist")
    lst.item().text("Wrote a Rust Library with C++ Bindings")


@slides.slide(debug_boxes=False)
def prefix(slide):
    text_area = text_slide(slide, "Prefix")
    lst = unordered_list(text_area)
    lst.item().text("No AI generation tools were used")
    lst.item().text("Slides and more at ~link{tylerjw.dev}")


@slides.slide(debug_boxes=False)
def why(slide):
    text_area = text_slide(slide, "What Are We Going To Cover")
    lst = unordered_list(text_area)
    lst.item().text("Social Objections to Rust")
    lst.item().text("Details of Interop")
    lst.item().text("Examples of Useful Patterns")
    lst.item().text("Code Generation Tools")


@slides.slide(debug_boxes=False)
def why(slide):
    text_area = text_slide(slide, "A Collective Craft")

    lead = text_area.sbox(p_bottom=40).text(
        "Quality of your project has more to do with the people\n"
        "that build it than the tools selected. It is fine\n"
        "and good for the people to like their tools.",
        style=elsie.TextStyle(align="middle"),
    )

    lst = unordered_list(text_area.box())
    lst.item().text("C++ code that exists has value")
    lst.item().text("A little Rust is better than no Rust")


@slides.slide(debug_boxes=False)
def why(slide):
    text_area = text_slide(slide, "After Action Report")
    text_area.sbox(p_bottom=40).text(
        "Coworker wrote this about writing a Rust project.",
        style=elsie.TextStyle(align="middle", bold=True),
    )
    lst = unordered_list(text_area.box())
    lst.item().text("It was quick! (2 engineers took 5 days)")
    lst.item().text("Cargo was a pleasure to work with.")
    lst.item().text(
        "It really helps focusing on the code instead\n  of dependencies / build rules."
    )
    lst.item().text("Going back to cmake / ament feels miserable.")
    lst.item().text("Builds are super quick.")
    lst.item().text("Compiler errors are helpful.")
    lst.item().text("Great vscode integration.")
    lst.item().text("Safe, modern and efficient at the core.")

@slides.slide(debug_boxes=False)
def first_class_types(slide):
    code_bg = "#EAEAEA"
    text_area = text_slide(slide, "Is this Possible?")
    text_area.sbox(p_bottom=40).text(
        "Rust",
        style=elsie.TextStyle(align="middle", bold=True),
    )
    code_block_1 = text_area.sbox(
        name="code_block_1",
        width="100%",
        z_level=-1,
    )
    code_block_1.rect(bg_color=code_bg, rx=20, ry=20)
    code_block_1.box(
        z_level=0,
        p_left=20,
        p_right=20,
        p_top=20,
        p_bottom=20,
        width="100%"
    ).code(
        "Rust",
        """let joint = Joint::new();
let transform = joint.calculate_transform(&[1.5]);
""",
    )
    text_area.sbox(p_top=60, p_bottom=40).text(
        "C++",
        style=elsie.TextStyle(align="middle", bold=True),
    )

    code_block_2 = text_area.sbox(
        name="code_block_2",
        width="100%",
        z_level=-1,
    )
    code_block_2.rect(bg_color=code_bg, rx=20, ry=20)
    code_block_2.box(
        z_level=0,
        p_left=20,
        p_right=20,
        p_top=20,
        p_bottom=20,
        width="100%"
    ).code(
        "C++",
        """Joint joint();
Eigen::Isometry3d transform = joint.calculate_transform(Eigen::VectorXd({1.5}));
""",
    )

@slides.slide(debug_boxes=False)
def bridge(slide):
    full_image_slide(
        slide, "Golden Gate Bridge", get_image_path("GoldenGateBridge.jpg")
    )


@slides.slide(debug_boxes=False)
def other_info(slide):
    text_area = text_slide(slide, "Before We Begin")
    text_area.sbox(p_bottom=40).text(
        "Code Generators",
        style=elsie.TextStyle(align="middle", bold=True),
    )
    lst = unordered_list(text_area.box())
    lst.item().text("cxx – Safe interop between Rust and C++")
    lst.item().text("bindgen – generate Rust FFI to C/C++ headers")
    lst.item().text("cbindgen – generate C headers for Rust FFI")
    text_area.sbox(p_top=60, p_bottom=40).text(
        "Why Not",
        style=elsie.TextStyle(align="middle", bold=True),
    )
    lst = unordered_list(text_area.box())
    lst.item().text("Eigen C++ types <=> Nalgebra Rust types")


@slides.slide(debug_boxes=False)
def bridge(slide):
    full_image_slide(
        slide, "Hourglass Language Bridge", get_image_path("hourglass_rust_cpp.png")
    )


@slides.slide(debug_boxes=False)
def layout(slide):
    code_slide(
        slide,
        "Project Layout",
        "",
        """├── Cargo.toml
├── README.md
└── crates
    ├── robot_joint
    │   ├── Cargo.toml
    │   └── src
    │       └── lib.rs
""",
    )


@slides.slide(debug_boxes=False)
def layout(slide):
    code_slide(
        slide,
        "Project Layout",
        "",
        """├── Cargo.toml
├── README.md
└── crates
    ├── robot_joint
    │   ├── Cargo.toml
    │   └── src
    │       └── lib.rs
    └── robot_joint-cpp
        ├── Cargo.toml
        ├── CMakeLists.txt
        ├── cmake
        │   └── robot_jointConfig.cmake.in
        ├── include
        │   └── robot_joint.hpp
        └── src
            ├── lib.cpp
            └── lib.rs
""",
    )

@slides.slide(debug_boxes=False)
def bridge(slide):
    full_image_slide(slide, "Zakim Bridge", get_image_path("Zakimbridge.jpg"))


@slides.slide(debug_boxes=False)
def opaque_types(slide):
    code_slide(
        slide,
        "robot_joint/src/lib.rs",
        "Rust",
        """pub struct Joint {
    name: String,
    parent_link_to_joint_origin: Isometry3<f64>,
}

impl Joint {
    pub fn new() -> Self;
}
""",
    )


@slides.slide(debug_boxes=False)
def opaque_types(slide):
    code_slide(
        slide,
        "robot_joint-cpp/src/lib.rs",
        "Rust",
        """use robot_joint::Joint;

#[no_mangle]
extern "C" fn robot_joint_new() -> *mut Joint {
    Box::into_raw(Box::new(Joint::new()))
}

#[no_mangle]
extern "C" fn robot_joint_free(joint: *mut Joint) {
    unsafe {
        drop(Box::from_raw(joint));
    }
}
""",
    )


@slides.slide(debug_boxes=False)
def opaque_types(slide):
    code_slide(
        slide,
        "robot_joint-cpp/include/robot_joint.hpp",
        "C++",
        """struct RustJoint;

class Joint {
  public:
    Joint();
    ~Joint();

    // Disable copy as we cannot safely copy opaque pointers to rust objects.
    Joint(Joint& other) = delete;
    Joint& operator=(Joint& other) = delete;

    // Explicit move.
    Joint(Joint&& other);
    Joint& operator=(Joint&& other);

  private:
    RustJoint* joint_ = nullptr;
};
""",
    )


@slides.slide(debug_boxes=False)
def opaque_types(slide):
    code_slide(
        slide,
        "robot_joint-cpp/src/lib.cpp",
        "C++",
        """#include "robot_joint.hpp"

extern "C" {
extern RustJoint* robot_joint_new();
extern void robot_joint_free(RustJoint*);
}
""",
    )


@slides.slide(debug_boxes=False)
def opaque_types(slide):
    code_slide(
        slide,
        "robot_joint-cpp/src/lib.cpp",
        "C++",
        """Joint::Joint() : joint_(robot_joint_new()) {}

Joint::~Joint() {
  if (joint_ != nullptr) {
    robot_joint_free(joint_);
  }
}

Joint::Joint(Joint&& other) : joint_(other.joint_) {
  other.joint_ = nullptr;
}

Joint& Joint::operator=(Joint&& other) {
  joint_ = other.joint_;
  other.joint_ = nullptr;
  return *this;
}
""",
    )


@slides.slide(debug_boxes=False)
def other_info(slide):
    text_area = text_slide(slide, "Build System Integration")
    text_area.sbox(p_bottom=40).text(
        "See My Blog for a link to CMake example\n" "~link{tylerjw.dev}",
        style=elsie.TextStyle(align="middle", bold=True),
    )


@slides.slide(debug_boxes=False)
def bridge(slide):
    full_image_slide(
        slide, "Fremont Bridge", get_image_path("Fremont_Bridge_Portland_Oregon.jpg")
    )


@slides.slide(debug_boxes=False)
def first_class_types(slide):
    code_bg = "#EAEAEA"
    text_area = text_slide(slide, "First-class Types")
    text_area.sbox(p_bottom=40).text(
        "robot_joint/src/lib.rs",
        style=elsie.TextStyle(align="middle", bold=True),
    )
    code_block_1 = text_area.sbox(
        name="code_block_1",
        width="100%",
        z_level=-1,
    )
    code_block_1.rect(bg_color=code_bg, rx=20, ry=20)
    code_block_1.box(
        z_level=0,
        p_left=20,
        p_right=20,
        p_top=20,
        p_bottom=20,
    ).code(
        "Rust",
        """impl Joint {
    pub fn calculate_transform(&self, variables: &[f64]) -> Isometry3<f64>;
}
""",
    )
    text_area.sbox(p_top=60, p_bottom=40).text(
        "robot_joint-cpp/include/robot_joint.hpp",
        style=elsie.TextStyle(align="middle", bold=True),
    )

    code_block_2 = text_area.sbox(
        name="code_block_2",
        width="100%",
        z_level=-1,
    )
    code_block_2.rect(bg_color=code_bg, rx=20, ry=20)
    code_block_2.box(
        z_level=0,
        p_left=20,
        p_right=20,
        p_top=20,
        p_bottom=20,
    ).code(
        "C++",
        """class Joint {
  public:
    Eigen::Isometry3d calculate_transform(const Eigen::VectorXd& variables);
};
""",
    )


@slides.slide(debug_boxes=False)
def first_class_types(slide):
    code_slide(
        slide,
        "robot_joint-cpp/src/lib.rs",
        "Rust",
        """#[repr(C)]
struct Mat4d {
    data: [c_double; 16],
}

#[no_mangle]
extern "C" fn robot_joint_calculate_transform(
    joint: *const Joint,
    variables: *const c_double,
    size: c_uint,
) -> Mat4d {
    unsafe {
        let joint = joint.as_ref().expect("Invalid pointer to Joint");
        let variables = std::slice::from_raw_parts(variables, size as usize);
        let transform = joint.calculate_transform(variables);
        Mat4d {
            data: transform.to_matrix().as_slice().try_into().unwrap(),
        }
    }
}
""",
    )


@slides.slide(debug_boxes=False)
def first_class_types(slide):
    code_slide(
        slide,
        "robot_joint-cpp/src/lib.cpp",
        "C++",
        """struct Mat4d {
  double data[16];
};

extern "C" {
extern struct Mat4d robot_joint_calculate_transform(
  const RustJoint*, const double*, unsigned int);
}

Eigen::Isometry3d Joint::calculate_transform(const Eigen::VectorXd& variables)
{
  const auto rust_isometry = robot_joint_calculate_transform(
    joint_, variables.data(), variables.size());
  Eigen::Isometry3d transform;
  transform.matrix() = Eigen::Map<Eigen::Matrix4d>(rust_isometry.data);
  return transform;
}
""",
    )


@slides.slide(debug_boxes=False)
def bridge(slide):
    full_image_slide(
        slide, "Red Cliff Bridge", get_image_path("Redcliff_bridge_2006.jpg")
    )


@slides.slide(debug_boxes=False)
def so_what(slide):
    text_area = text_slide(slide, "So What?")
    text_area.sbox(p_bottom=40).text(
        "Rust / C++ Interop is Straightforward\nDon’t Listen to the Naysayers",
        style=elsie.TextStyle(align="middle", bold=True),
    )


@slides.slide(debug_boxes=False)
def creddits(slide):
    text_area = text_slide(slide, "Attribution")
    text_area.sbox(p_bottom=40).text(
        "Kyle Cesare's OptIk\n" "~link{github.com/kylc/optik}",
        style=elsie.TextStyle(align="middle", bold=True),
    )


render_deck(slides, "rust_cpp_interop.pdf", page_numbering_func=my_page_numbering)
