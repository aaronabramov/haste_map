const HasteMap = require("jest-haste-map");
const path = require("path");
const os = require("os");

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
});
