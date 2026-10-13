// Run against actual wasm-bindgen Node output; an override supports isolated builds.
const assert = require('node:assert/strict');
const {Session} = require(process.env.GENSOLID_WASM_MODULE || '../pkg/gensolid_wasm.js');

function check(session, body, bounds) {
    assert.equal(session.bodyCheck(body), 0);
    assert.equal(session.bodyAskFaces(body).length, 7);
    assert.equal(session.bodyAskEdges(body).length, 14);
    const vertices = session.bodyAskVertices(body);
    assert.equal(vertices.length, 8);
    const actual = Array.from(vertices, v => JSON.stringify(Array.from(session.vertexAskPoint(v)))).sort();
    const expected = [];
    for (const x of bounds[0]) for (const y of bounds[1]) for (const z of bounds[2])
        expected.push(JSON.stringify([x, y, z]));
    assert.deepEqual(actual, expected.sort());
}

const s = new Session();
try {
    for (const [r, offset] of [[4, [16, 0]], [0, [0, 0]], [Infinity, [0, 0]], [4, [NaN, 0]]])
        assert.throws(() => s.bodyCreateSolidBlockWithOffsetHole(40, 25, 8, r, offset));
    assert.throws(() => s.bodyCreateSolidBlockWithOffsetHole(40, 25, 8, 4, [6]), /2 numbers/);
    assert.throws(() => s.bodyCreateSolidBlockWithHoleAt(40, 25, 8, 4, [0, 0], [0, 0, 1], [1, 0, 0]), /3 numbers/);
    assert.throws(() => s.bodyCreateSolidBlockWithHoleAt(40, 25, 8, 4, [0, 0, 0], [0, 0, 0], [1, 0, 0]));
    const first = s.bodyCreateSolidBlockWithHole(40, 25, 8, 4);
    assert.equal(first, 1);
    check(s, first, [[-20, 20], [-12.5, 12.5], [-4, 4]]);
    check(s, s.bodyCreateSolidBlockWithOffsetHole(40, 25, 8, 4, [6, -3]), [[-20, 20], [-12.5, 12.5], [-4, 4]]);
    for (const body of [
        s.bodyCreateSolidBlockWithHoleAt(40, 25, 8, 4, [10, 20, 30], [1, 0, 0], [0, 1, 0]),
        s.bodyCreateSolidBlockWithOffsetHoleAt(40, 25, 8, 4, [6, -3], [10, 20, 30], [1, 0, 0], [0, 1, 0]),
    ]) check(s, body, [[6, 14], [0, 40], [17.5, 42.5]]);
    assert.throws(() => s.bodyCheck(999));
    assert.throws(() => s.vertexAskPoint(first));
    console.log('WASM: all through-hole contracts passed');
} finally { s.free(); }
