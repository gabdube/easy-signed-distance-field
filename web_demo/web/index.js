import init, { startup, render, parse_font, update_render_mid_value, compute_fixed_height } from "./app/app.js";

let ctxOutput, ctxRender, ctxNative;

async function loadDefaultFont() {
    let request = await fetch("./Questrial-Regular.ttf");
    
    let data = await request.arrayBuffer();
    parse_font(data);


    let font = new FontFace("custom", data);
    await font.load();
    document.fonts.add(font);
    updateDemo();
}

async function loadFont() {
    let fontInput = document.getElementById("textFont");
    if (fontInput.files.length == 0) {
        loadDefaultFont();
        return;
    }

    let data = await fontInput.files[0].arrayBuffer();
    parse_font(data);

    let font = new FontFace("custom", data);
    await font.load();
    document.fonts.add(font);
    updateDemo();
}

function updateDemo() {
    let character = document.getElementById("textValue").value[0];
    if (!character) { return; }

    let size = document.getElementById("textSize").value;
    let spread = document.getElementById("textSpread").value;

    ctxNative.font = `500px 'custom'`;
    let metrics = ctxNative.measureText(character);
    let width = metrics.actualBoundingBoxLeft + metrics.actualBoundingBoxRight;
    let height = metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent;

    ctxOutput.canvas.width = width;
    ctxOutput.canvas.height = height; 
    ctxOutput.canvas.style.maxWidth = `${width}px`;
    ctxOutput.canvas.style.maxHeight =`${height}px`;

    ctxRender.canvas.width = width;
    ctxRender.canvas.height = height;
    ctxRender.canvas.style.maxWidth = `${width}px`;
    ctxRender.canvas.style.maxHeight =`${height}px`;
    
    ctxNative.canvas.width = width;
    ctxNative.canvas.height = height;
    ctxNative.canvas.style.maxWidth = `${width}px`;
    ctxNative.canvas.style.maxHeight =`${height}px`;

    ctxNative.font = `500px 'custom'`;
    ctxNative.fillText(character, metrics.actualBoundingBoxLeft, metrics.actualBoundingBoxAscent);

    if (useFixedHeight()) {
        size = compute_fixed_height(character, size);
    }

    render(
        ctxOutput,
        character,
        size,
        spread,
    );
}

function updateMidValue() {
    const clamp = (num, min, max) => Math.min(Math.max(num, min), max);
    const v = clamp(document.getElementById("textMidValue").value, 0.0, 1.0);
    update_render_mid_value(v);
    updateDemo();
}

function useFixedHeight() {
    return document.getElementById("fixedHeight").checked;
}

function updateFixedHeight() {
    const checked = useFixedHeight();
    const sizeLabel = document.getElementById("sizeLabel");
    if (checked) {
        sizeLabel.innerHTML = "Sdf height (in px): "
    } else {
        sizeLabel.innerHTML = "Sdf font size (in px): "
    }

    updateDemo();
}

async function init_demo() {
    await init();

    let canvasOutput = document.getElementById("canvasOutput");
    ctxOutput = canvasOutput.getContext('2d');

    let canvasRender = document.getElementById("canvasRender");
    ctxRender = canvasRender.getContext('webgl2');

    let canvasNative = document.getElementById("canvasNative");
    ctxNative = canvasNative.getContext('2d');

    document.getElementById("textValue").addEventListener("input", updateDemo);
    document.getElementById("textSize").addEventListener("change", updateDemo);
    document.getElementById("textSpread").addEventListener("change", updateDemo);
    document.getElementById("textFont").addEventListener("change", loadFont);
    document.getElementById("textMidValue").addEventListener("change", updateMidValue);
    document.getElementById("fixedHeight").addEventListener("change", updateFixedHeight);

    startup(ctxRender);

    await loadFont();
    updateMidValue();
    updateFixedHeight();

}

init_demo();
