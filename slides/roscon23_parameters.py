#!/usr/bin/env python3

# TODO:
# - qr codes!
# - What is the call to action? What should people do use / contribute / upstream?
# - How do we theme the presentation? Jokes? Capture / Hold attention?

import elsie
from elsie.ext import unordered_list
from elsie.boxtree.box import Box
from my_layouts import *


slides = init_deck()


@slides.slide(debug_boxes=False)
def title(slide):
    content = logo_header_slide(slide, "")
    content.box(width="fill").text(
        "Parameters Should\nBe Boring", elsie.TextStyle(size=160, bold=True)
    )
    content.box(width="fill").text(
        "generate_parameter_library", elsie.TextStyle(size=36, bold=True, italic=True)
    )
    content.box(width="fill", p_top=10).text("October 20, 2023")
    content.box(width="fill", p_top=180).text("Tyler Weaver")
    content.box(width="fill").text("Staff Software Engineer\ntyler@picknik.ai")


@slides.slide(debug_boxes=False)
def author(slide):
    text_area = image_slide(slide, "Tyler Weaver", get_image_path("kart.jpg"))
    lst = unordered_list(text_area)
    lst.item().text("Racing Kart Driver")
    lst.item().text("MoveIt Maintainer")
    lst.item().text("Rust Evangelist")
    lst.item().text("Docker Skeptic")


@slides.slide(debug_boxes=False)
def part_1(slide):
    section_title_slide(slide, "RCLCPP\nParameters", "Part 1")


@slides.slide(debug_boxes=False)
def innocence(slide):
    grayed_before_after_code_slide(
        slide,
        "Getting Started",
        "C++",
        """int main(int argc, char ** argv)
{
  rclcpp::init(argc, argv);

  auto node = std::make_shared<rclcpp::Node>("minimal_param_node");
""",
        """
  auto my_string = node->declare_parameter("my_string", "world");
  auto my_number = node->declare_parameter("my_number", 23);
""",
        """
  rclcpp::spin(node);
  rclcpp::shutdown();
}
""",
    )


@slides.slide(debug_boxes=False)
def struct_with_defaults(slide):
    code_slide(
        slide,
        "Parameter Struct",
        "C++",
        """
struct Params {
  std::string my_string = "world";
  int my_number = 23;
};

int main(int argc, char ** argv)
{
  rclcpp::init(argc, argv);
  auto node = std::make_shared<rclcpp::Node>("minimal_param_node");
  auto params = Params{};
  params.my_string = node->declare_parameter("my_string", params.my_string);
  params.my_number = node->declare_parameter("my_number", params.my_number);

  rclcpp::spin(node);
  rclcpp::shutdown();
}
""",
    )


@slides.slide(debug_boxes=False)
def details(slide):
    code_slide(
        slide,
        "ParameterDescriptor",
        "C++",
        """
int main(int argc, char ** argv)
{
  rclcpp::init(argc, argv);
  auto node = std::make_shared<rclcpp::Node>("minimal_param_node");
  auto params = Params{};

  auto param_desc  = rcl_interfaces::msg::ParameterDescriptor{};
  param_desc.description = "Mine!";
  param_desc.additional_constraints  = "One of [world, base, home]";
  params.my_string = node->declare_parameter("my_string",
    params.my_string, param_desc);

  param_desc  = rcl_interfaces::msg::ParameterDescriptor{};
  param_desc.description = "Who controls the universe?";
  param_desc.additional_constraints  = "A multiple of 23";
  params.my_number = node->declare_parameter("my_number",
    params.my_number, param_desc);
  //...
""",
    )


@slides.slide(debug_boxes=False)
def validate(slide):
    code_slide(
        slide,
        "Validation",
        "C++",
        """
auto const _ = node->add_on_set_parameters_callback(
  [](std::vector<rclcpp::Parameter> const& params)
  {
    for (auto const& param : params) {
      if(param.get_name() == "my_string") {
          auto const value = param.get_value<std::string>();
          auto const valid = std::vector<std::string>{"world", "base", "home"};
          if (std::find(valid.cbegin(), valid.cend(), value) == valid.end()) {
            auto result = rcl_interfaces::msg::SetParametersResult{};
            result.successful = false;
            result.reason = std::string("my_string: {")
              .append(value)
              .append("} not one of: [world, base, home]");
            return result;
          }
        }
      }
    return rcl_interfaces::msg::SetParametersResult{};
  });
""",
    )


@slides.slide(debug_boxes=False)
def copy_pasta(slide):
    content = logo_header_slide(slide, "Copy Pasta")
    lst = unordered_list(content.box())
    lst.item().text("parameter name: 6 separate copies")
    lst.item().text("declaration: re-init description for each parameter")
    lst.item().text("validation: convert vector to map")


@slides.slide(debug_boxes=False)
def validate(slide):
    content = logo_header_slide(slide, "")
    content.box().text("30 lines of C++ boilderpate per parameter")
    content.box(show="2").text(
        "Before handling of dynamic parameters", elsie.TextStyle(color="red")
    )


@slides.slide(debug_boxes=False)
def part2(slide):
    section_title_slide(slide, "generate_\nparameter_library", "Part 2")


@slides.slide(debug_boxes=False)
def gpl(slide):
    code_slide(
        slide,
        "YAML",
        "toml",  # I know this is yaml, parser doesn't like it though
        """
minimal_param_node:
    my_string: {
        type: string,
        description: "Mine!"
        validation: {
            one_of<>: [["world", "base", "home"]]
        }
    }
    my_number: {
        type: int
        description: "Mine!"
        validation: {
            multiple_of_23: []
        }
    }
""",
    )


@slides.slide(debug_boxes=False)
def gpl(slide):
    code_slide(
        slide,
        "CMake Module",
        "cmake",
        """
find_package(generate_parameter_library REQUIRED)

generate_parameter_library(
  minimal_param_node_parameters
  src/minimal_param_node.yaml
)

add_executable(minimal_node src/minimal_param_node.cpp)
target_link_libraries(minimal_node PRIVATE
  rclcpp::rclcpp
  minimal_param_node_parameters
)
""",
    )


@slides.slide(debug_boxes=False)
def gpl(slide):
    code_slide(
        slide,
        "C++ Usage",
        "C++",
        """
#include <rclcpp/rclcpp.hpp>
#include "minimal_param_node_parameters.hpp"

int main(int argc, char * argv[])
{
  rclcpp::init(argc, argv);
  auto node = std::make_shared<rclcpp::Node>("minimal_param_node");
  auto param_listener =
    std::make_shared<minimal_param_node::ParamListener>(node);
  auto params = param_listener->get_params();

  // ...
""",
    )


@slides.slide(debug_boxes=False)
def gpl(slide):
    content = logo_header_slide(slide, "Built-In Validation Functions")
    lst = unordered_list(content)
    lst.item().text("bounds (inclusive)")
    lst.item().text("less than")
    lst.item().text("greater than")
    lst.item().text("less than or equal")
    lst.item().text("greater than or equal")
    lst.item(show="2").text("fixed string/array length")
    lst.item(show="2").text("size of string/array length greater than")
    lst.item(show="2").text("size of string/array length less than")
    lst.item(show="2").text("array contains no duplicates")
    lst.item(show="2").text("array is a subset of another array")
    lst.item(show="2").text("bounds checking for elements of an array")


@slides.slide(debug_boxes=False)
def gpl(slide):
    code_slide(
        slide,
        "Custom Validation",
        "C++",
        """
#include <rclcpp/rclcpp.hpp>
#include <fmt/core.h>
#include <tl_expected/expected.hpp>

tl::expected<void, std::string> multiple_of_23(
    rclcpp::Parameter const& parameter) {
  int param_value = parameter.as_int();
    if (param_value % 23 != 0) {
        return tl::make_unexpected(fmt::format(
            "Invalid value '{}' for parameter {}. Must be multiple of 23.",
            param_value, parameter.get_name());
    }
  return {};
}
""",
    )


@slides.slide(debug_boxes=False)
def gpl(slide):
    content = logo_header_slide(slide, "Other Killer Features")
    lst = unordered_list(content.box())
    lst.item().text("Dynamic parameters")
    lst.item().text("Generation of RCLPY Parameter Libraries")
    lst.item().text("Generation of Markdown Docs")
    lst.item().text(
        "Examples and docs at\n~link{github.com/PickNikRobotics/generate_parameter_library}"
    )


@slides.slide(debug_boxes=False)
def part3(slide):
    section_title_slide(slide, "Boring?", "Part 3")


@slides.slide(debug_boxes=False)
def gpl(slide):
    content = logo_header_slide(slide, "Why so many parameters?")
    lst = unordered_list(content.box())
    lst.item().text("Users use defaults for most parameters")
    lst.item(show="2+").text("Authors only test default values")
    lst.item(show="3+").text("Permutations of parameters grow exponentially")
    lst.item(show="4+").text(
        "The more complex your interface the less useful your abstraction"
    )
    lst.item(show="5+").text("Resist the urge to expose interior details as parameters")


@slides.slide(debug_boxes=False)
def gpl(slide):
    content = logo_header_slide(slide, "What is a good parameter?")
    lst = unordered_list(content.box())
    lst.item().text("Express user intent (latency or throughput)")
    lst.item(show="2+").text("Details like buffer sizes scale with hardware")
    lst.item(show="3+").text(
        "Leave the door open to improvements in behavior for the user"
    )


@slides.slide(debug_boxes=False)
def thank_you(slide):
    content = logo_header_slide(slide, "")
    content.fbox().text(
        "Questions?\n~link{github.com/PickNikRobotics/generate_parameter_library}\n",
        elsie.TextStyle(align="middle"),
    )
    content.sbox(p_bottom=20).text(
        "Slides generated using Elsie\n~link{tylerjw.dev/roscon23-parameters/}",
        elsie.TextStyle(align="right", size=32),
    )


render_deck(slides, "roscon23_parameters.pdf")
