"""Verify the generated HTML offline in a headless Chromium browser and save its preview."""

import argparse
import csv
import os
from pathlib import Path

from playwright.sync_api import sync_playwright


def csv_numbers(path):
    with path.open(encoding="utf-8", newline="") as src:
        reader = csv.reader(src)
        return next(reader), [list(map(float, row)) for row in reader]


def run(folder, executable):
    with sync_playwright() as pw:
        browser = pw.chromium.launch(executable_path=executable, headless=True)
        context = browser.new_context(
            viewport={"width": 1440, "height": 960},
            reduced_motion="reduce",
            offline=True,
        )
        page = context.new_page()
        errors, remote = [], []
        page.on("pageerror", lambda error: errors.append(str(error)))
        page.on(
            "request",
            lambda req: (
                remote.append(req.url)
                if req.url.startswith(("http:", "https:"))
                else None
            ),
        )
        page.goto((folder / "index.html").as_uri())
        page.wait_for_selector("html[data-sweep-ready=true]")
        assert page.locator("#order option").count() == 17
        page.locator("#before").click()
        assert page.locator("#value").text_content() == "0,135"
        assert page.locator("#fieldCount").text_content().endswith("8")
        page.locator("#after").click()
        assert page.locator("#value").text_content() == "4,309"
        assert page.locator("#fieldCount").text_content().endswith("6")
        assert page.locator("#order").input_value() == "3"
        page.locator("#order").select_option("16")
        assert page.locator("#value").text_content() == "—"
        page.locator("#prev").click()
        assert page.locator("#order").input_value() == "15"
        page.locator("#next").click()
        assert page.locator("#next").is_disabled()
        page.locator("#after").click()
        page.locator("#field").focus()
        page.keyboard.press("ArrowLeft")
        assert page.locator("#angle").input_value() == "312"
        page.keyboard.press("Space")
        page.wait_for_function("Number(document.getElementById('angle').value)>314")
        page.locator("#play").click()
        assert page.locator("#play").get_attribute("aria-pressed") == "false"
        field = page.locator("#field")
        box = field.bounding_box()
        field.click(position={"x": box["width"] * 0.79, "y": box["height"] * 0.505})
        assert (
            min(
                int(page.locator("#angle").input_value()),
                1440 - int(page.locator("#angle").input_value()),
            )
            < 3
        )
        heat = page.locator("#heat")
        box = heat.bounding_box()
        heat.click(position={"x": box["width"] * 0.65, "y": box["height"] * 0.45})
        assert 5 <= int(page.locator("#order").input_value()) <= 10
        page.locator("#after").click()
        original = field.screenshot()
        page.locator("#centers").check()
        assert field.screenshot() != original
        polar = page.locator("#polar")
        original = polar.screenshot()
        page.locator("#allPolars").check()
        assert polar.screenshot() != original
        page.locator("#allPolars").uncheck()
        page.locator("#centers").uncheck()
        for control, file in [
            ("saveSegments", "segments.csv"),
            ("saveCenters", "centroids.csv"),
        ]:
            with page.expect_download() as info:
                page.locator("#" + control).click()
            assert csv_numbers(info.value.path()) == csv_numbers(folder / file)
        page.screenshot(path=folder / "preview.png", full_page=True)
        page.set_viewport_size({"width": 390, "height": 844})
        assert page.evaluate("document.documentElement.scrollWidth <= innerWidth")
        page.emulate_media(reduced_motion="no-preference")
        page.reload()
        page.wait_for_function("Number(document.getElementById('angle').value)>119")
        assert not remote and not errors, (remote, errors)
        browser.close()
        print(
            "PASS offline UI, 17 ranks, tangent, playback, pointer/keyboard controls, two layers, two exact CSV exports, mobile viewport"
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).resolve().parents[2] / "target" / "spectre-sweep",
    )
    default_edge = (
        Path(os.getenv("ProgramFiles(x86)", ""))
        / "Microsoft/Edge/Application/msedge.exe"
    )
    parser.add_argument(
        "--browser", default=str(default_edge) if default_edge.exists() else None
    )
    args = parser.parse_args()
    run(args.output.resolve(), args.browser)
