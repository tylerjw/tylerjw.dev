import elsie
from elsie.boxtree.box import Box


def init_slides():
    slides = elsie.SlideDeck(width=1920, height=1080)
    slides.update_style("default", elsie.TextStyle(font="Lato", align="left", size=64))
    slides.update_style("code", elsie.TextStyle(size=38))
    slides.set_style("link", elsie.TextStyle(color="blue"))
    return slides


def get_image_path(filename: str):
    images_dir = "../../static/images/"
    return images_dir + filename


def logo_header_slide(parent: Box, title: str):
    parent.box(x=1570, y=40).image(get_image_path("picknik_logo.png"))
    parent.sbox(name="header", x=0, height=140).fbox(p_left=20).text(
        title, elsie.TextStyle(bold=True)
    )
    return parent.fbox(name="content", p_left=20, p_right=20)


def image_slide(parent: Box, title: str, image_path: str):
    content = logo_header_slide(parent, title)
    content = content.fbox(horizontal=True, p_top=20, p_bottom=20)
    text_area = content.fbox(name="text_area", width="50%")
    content.sbox(name="image", width="fill").image(image_path)
    return text_area


def section_title_slide(parent: Box, title: str, subtitle: str):
    content = logo_header_slide(parent, "")
    content.sbox().text(title, elsie.TextStyle(align="right", size=240, bold=True))
    content.box().text(subtitle)


def code_slide(parent: Box, title: str, language: str, code: str):
    content = logo_header_slide(parent, title)
    code_bg = "#F6F8FA"
    box = content.box(y=0, width="100%", height="100%", p_bottom=20)
    box.rect(bg_color=code_bg, rx=20, ry=20)
    box.overlay().box(x=0, y=0, p_left=20, p_right=20, p_top=20, p_bottom=20).code(
        language, code
    )
