// generate-icons.js
import sharp from 'sharp';
import sharpIco from 'sharp-ico';
import { existsSync, mkdirSync } from 'node:fs';

const sizes = [32, 128];
const squareSizes = [30, 44, 71, 89, 107, 142, 150, 284, 310];
const logoPath = 'src/assets/logo-square.svg';
const iconsTargetPath = 'src-tauri/icons';

(async () => {
  if (!existsSync('src-tauri/icons')) {
    mkdirSync('src-tauri/icons', { recursive: true });
  }

  for (const size of sizes) {
    await sharp(logoPath).resize(size, size).png().toFile(`${iconsTargetPath}/${size}x${size}.png`);
  }

  // 128x128@2x.png
  await sharp(logoPath).resize(256, 256).png().toFile(`${iconsTargetPath}/128x128@2x.png`);

  // icon.png
  await sharp(logoPath).resize(512, 512).png().toFile(`${iconsTargetPath}/icon.png`);

  for (const size of squareSizes) {
    await sharp(logoPath).resize(size, size).png().toFile(`${iconsTargetPath}/Square${size}x${size}Logo.png`);
  }

  // StoreLogo.png
  await sharp(logoPath).resize(50, 50).png().toFile(`${iconsTargetPath}/StoreLogo.png`);

  // icon.ico
  await sharpIco.sharpsToIco([sharp(logoPath).resize(16, 16)], `${iconsTargetPath}/icon.ico`);
})();
