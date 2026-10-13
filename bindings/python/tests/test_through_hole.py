"""Exercise the installed extension, including frame and failure contracts."""
import itertools
import math
import unittest

import gensolid


class ThroughHoleTests(unittest.TestCase):
    def assert_body(self, session, body, bounds):
        self.assertEqual(session.body_check(body), [])
        self.assertEqual(len(session.body_ask_faces(body)), 7)
        self.assertEqual(len(session.body_ask_edges(body)), 14)
        vertices = session.body_ask_vertices(body)
        self.assertEqual(len(vertices), 8)
        points = {session.vertex_ask_point(v) for v in vertices}
        self.assertEqual(points, set(itertools.product(*bounds)))

    def test_centred_and_offset_plate(self):
        session = gensolid.Session()
        for body in [
            session.body_create_solid_block_with_hole(40, 25, 8, 4, location=[0, 0, 4]),
            session.body_create_solid_block_with_offset_hole(
                40, 25, 8, 4, [6, -3], location=[0, 0, 4]
            ),
        ]:
            self.assert_body(session, body, [(-20, 20), (-12.5, 12.5), (0, 8)])

    def test_rotated_frame(self):
        session = gensolid.Session()
        for create, args in [
            (session.body_create_solid_block_with_hole, (40, 25, 8, 4)),
            (session.body_create_solid_block_with_offset_hole, (40, 25, 8, 4, [6, -3])),
        ]:
            body = create(*args, location=[10, 20, 30], axis=[1, 0, 0], ref_direction=[0, 1, 0])
            self.assert_body(session, body, [(6, 14), (0, 40), (17.5, 42.5)])

    def test_rejected_input_does_not_consume_tag(self):
        session = gensolid.Session()
        for radius, offset in [(4, [16, 0]), (0, [0, 0]), (4, [math.nan, 0]), (math.inf, [0, 0])]:
            with self.assertRaises(ValueError):
                session.body_create_solid_block_with_offset_hole(40, 25, 8, radius, offset)
        with self.assertRaises(ValueError):
            session.body_create_solid_block_with_hole(40, 25, 8, 4, axis=[0, 0, 0])
        self.assertEqual(session.body_create_solid_block_with_hole(40, 25, 8, 4), 1)

    def test_array_and_tag_errors(self):
        session = gensolid.Session()
        with self.assertRaises((TypeError, ValueError)):
            session.body_create_solid_block_with_offset_hole(40, 25, 8, 4, [6])
        with self.assertRaises((TypeError, ValueError)):
            session.body_create_solid_block_with_hole(40, 25, 8, 4, location=[0, 0])
        with self.assertRaises(KeyError):
            session.body_check(999)
        body = session.body_create_solid_block_with_hole(40, 25, 8, 4)
        with self.assertRaises(TypeError):
            session.vertex_ask_point(body)


if __name__ == '__main__':
    unittest.main()
