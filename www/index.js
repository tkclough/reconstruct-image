import * as wasm from "reconstruct-image"

let image_reconstruction;

let original = document.getElementById("original-image"),
    grayscale = document.getElementById("grayscale-image"),
    corrupted = document.getElementById("corrupted-image"),
    reconstructed = document.getElementById("reconstructed-image");

let originalCanvas = document.createElement("canvas"),
    grayscaleCanvas = document.createElement("canvas"),
    corruptedCanvas = document.createElement("canvas"),
    reconstructedCanvas = document.createElement("canvas");

let originalCtx = originalCanvas.getContext("2d"),
    grayscaleCtx = grayscaleCanvas.getContext("2d"),
    corruptedCtx = corruptedCanvas.getContext("2d"),
    reconstructedCtx = reconstructedCanvas.getContext("2d");

function reset() {
  let probBar = document.getElementById("prob");
  let p = parseFloat(probBar.value);
  let data = originalCtx.getImageData(0, 0, originalCanvas.width, originalCanvas.height);
  image_reconstruction = wasm.ImageReconstruction.new(data, p);

  image_reconstruction.draw_original_image(grayscaleCtx);
  image_reconstruction.draw_corrupted_image(corruptedCtx);
  image_reconstruction.draw_reconstructed_image(reconstructedCtx);

  grayscale.src = grayscaleCanvas.toDataURL();
  corrupted.src = corruptedCanvas.toDataURL();
  reconstructed.src = reconstructedCanvas.toDataURL();
}

function draw() {
  let w = this.width,
      h = this.height;

  originalCanvas.width = w;
  originalCanvas.height = h;
  originalCtx.drawImage(this, 0, 0);

  grayscaleCanvas.width = w;
  grayscaleCanvas.height = h;

  corruptedCanvas.width = w;
  corruptedCanvas.height = h;
  
  reconstructedCanvas.width = w;
  reconstructedCanvas.height = h;

  reset();
}

function gradientStep(i) {
  let eta = etaInput.value;
  let steps = parseFloat(stepsPerClickRange.value);
  console.log(`${i}/${steps}`);
  if (i >= steps) return;

  image_reconstruction.gradient_step(eta);

  image_reconstruction.draw_reconstructed_image(reconstructedCtx);

  reconstructed = document.getElementById("reconstructed-image");
  reconstructed.src = reconstructedCanvas.toDataURL();
  
  reconstructed.onload = () => {
    gradientStep(i + 1);
  }
}
let gradientStepBtn = document.getElementById("gradient-step-btn");
let stepsPerClickRange = document.getElementById("steps-per-click");
let etaInput = document.getElementById("eta");

gradientStepBtn.onclick = () => gradientStep(0);

let resetBtn = document.getElementById("reset-btn");
resetBtn.onclick = reset;

function failed() {
  console.error("The provided file couldn't be loaded as an Image media");
}

function loadImage() {
  original.onload = draw;
  original.onerror = failed;
  original.src = URL.createObjectURL(this.files[0]);
}
let imageInput = document.getElementById("image-input");
imageInput.onchange = loadImage;
if (imageInput.files.length > 0) {
  loadImage.bind(imageInput)();
}