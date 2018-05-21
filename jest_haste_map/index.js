const HasteMap = require("jest-haste-map");
const path = require("path");
const os = require("os");
const fs = require("fs");

const DEPENDENCY_INDEX = 3;
const ARTIFACT_PATH = "../haste_map_js.txt";

const hasteMap = new HasteMap({
  name: "test",
  extensions: ["js"],
  ignorePattern: /node_modules/,
  platforms: ["ios", "android"],
  roots: [
    path.resolve(os.homedir(), "fbsource/xplat")
    // path.resolve(os.homedir(), 'fbsource/xplat/nuclide'),
  ]
});

console.log(HasteMap.getCacheFilePath(os.tmpdir(), "test"));
console.time("TOP LEVEL TIME WRAPPER");
hasteMap.build().then(m => {
  console.timeEnd("TOP LEVEL TIME WRAPPER");
  console.log(Object.keys(m.hasteFS._files).length);

  if (fs.existsSync(ARTIFACT_PATH)) {
    console.log("Artifact already exists");
  } else {
    console.log("Artifact not found creating...");
    const files = m.hasteFS._files;
    const artifact = Object.keys(files)
      .map(file => `${file}|${files[file][DEPENDENCY_INDEX].sort().join("|")}`)
      .sort()
      .join("\n");

    fs.writeFileSync(ARTIFACT_PATH, artifact);
  }
});
