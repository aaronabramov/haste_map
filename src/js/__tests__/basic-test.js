const path = require('path');
const hasteMap = require('../');

test('test', () => {
  const fixturesPath = path.resolve(
    __dirname,
    '../../../../../fbsource/xplat/nuclide',
  );
  console.time('hasteMap');
  const map = hasteMap.buildHasteMap(fixturesPath);
  console.timeEnd('hasteMap');
  console.log('entries:', Object.values(map).length);
});
