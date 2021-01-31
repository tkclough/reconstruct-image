import * as wasm from "reconstruct-image"

let image_reconstruction;

let originalCanvas = document.getElementById("original-image-canvas"),
    grayscaleCanvas = document.getElementById("grayscale-image-canvas"),
    corruptedCanvas = document.getElementById("corrupted-image-canvas"),
    reconstructedCanvas = document.getElementById("reconstructed-image-canvas");

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

  // let canvas = document.getElementById("original-image-canvas");
  // canvas.width = this.width;
  // canvas.height = this.height;

  // ctx.drawImage(this, 0, 0);

  // let data = ctx.getImageData(0, 0, canvas.width, canvas.height);
  // console.log(data);

  // let probBar = document.getElementById("prob");
  // let p = parseFloat(probBar.value);
  // image_reconstruction = wasm.ImageReconstruction.new(data, p);
  
  // let canvas2 = document.getElementById("grayscale-image-canvas");
  // canvas2.width = this.width;
  // canvas2.height = this.height;
  // let ctx2 = canvas2.getContext("2d");
  // image_reconstruction.draw_original_image(ctx2);

  // let canvas3 = document.getElementById("corrupted-image-canvas");
  // canvas3.width = this.width;
  // canvas3.height = this.height;
  // let ctx3 = canvas3.getContext("2d");
  // image_reconstruction.draw_corrupted_image(ctx3);

  // let canvas4 = document.getElementById("reconstructed-image-canvas");
  // canvas4.width = this.width;
  // canvas4.height = this.height;
  // let ctx4 = canvas4.getContext("2d");
  // image_reconstruction.draw_reconstructed_image(ctx4);
}

let button = document.getElementById("gradient-step");
button.onclick = function(ev) {
  let stepsPerClickBtn = document.getElementById("steps-per-click");
  let etaField = document.getElementById("eta");
  let eta = parseFloat(etaField.value);
  console.log("eta =", eta);
  console.log(stepsPerClickBtn.value);
  let canvas4 = document.getElementById("reconstructed-image-canvas");
  let ctx4 = canvas4.getContext("2d");
  for (let i = 0; i < stepsPerClickBtn.value; i++) {
    console.log(`${i + 1}/${stepsPerClickBtn.value}`);
    image_reconstruction.gradient_step(eta);
  }
  image_reconstruction.draw_reconstructed_image(ctx4);
}

let resetBtn = document.getElementById("reset-btn");
resetBtn.onclick = reset;

function failed() {
  console.error("The provided file couldn't be loaded as an Image media");
}

document.getElementById("image-input").onchange = function(event) {
  let img = new Image();
  img.onload = draw;
  img.onerror = failed;
  img.src = URL.createObjectURL(this.files[0]);
};