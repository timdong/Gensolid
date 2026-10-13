/* Real C ABI test: versioned options, no allocation on rejection, memory ownership. */
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include "gensolid.h"

static void check_body(GS_BODY_t body, const double bounds[3][2]) {
    int32_t n = -1;
    GS_FACE_t *faces = NULL;
    assert(GS_BODY_ask_faces(body, &n, &faces) == GS_ERROR_no_errors);
    assert(n == 7);
    assert(GS_MEMORY_free(faces) == GS_ERROR_no_errors);
    GS_EDGE_t *edges = NULL;
    assert(GS_BODY_ask_edges(body, &n, &edges) == GS_ERROR_no_errors);
    assert(n == 14);
    assert(GS_MEMORY_free(edges) == GS_ERROR_no_errors);
    GS_VERTEX_t *vertices = NULL;
    assert(GS_BODY_ask_vertices(body, &n, &vertices) == GS_ERROR_no_errors);
    assert(n == 8);
    unsigned seen = 0;
    for (int32_t i = 0; i < n; ++i) {
        double p[3];
        assert(GS_VERTEX_ask_point(vertices[i], &p) == GS_ERROR_no_errors);
        unsigned corner = 0;
        for (int j = 0; j < 3; ++j) {
            /* Orthogonal frame and half sizes are exactly representable. */
            assert(p[j] == bounds[j][0] || p[j] == bounds[j][1]);
            if (p[j] == bounds[j][1]) corner |= 1u << j;
        }
        assert((seen & (1u << corner)) == 0);
        seen |= 1u << corner;
    }
    assert(seen == 255);
    assert(GS_MEMORY_free(vertices) == GS_ERROR_no_errors);
    GS_check_fault_t *faults = NULL;
    assert(GS_BODY_check(body, &n, &faults) == GS_ERROR_no_errors);
    assert(n == 0 && faults == NULL);
    assert(GS_MEMORY_free(faults) == GS_ERROR_no_errors);
}

int main(void) {
    GS_BODY_t body = -1;
    assert(GS_BODY_create_solid_block_with_hole(40, 25, 8, 4, NULL, NULL, &body) == GS_ERROR_session_not_started);
    assert(body == -1);
    assert(GS_SESSION_start(NULL) == GS_ERROR_no_errors);
    assert(GS_BODY_create_solid_block(40, 25, 8, NULL, NULL) == GS_ERROR_null_argument);
    assert(GS_BODY_create_solid_block_with_hole(40, 25, 8, 4, NULL, NULL, NULL) == GS_ERROR_null_argument);
    assert(GS_BODY_create_solid_block_with_hole_o_init(NULL) == GS_ERROR_null_argument);
    GS_BODY_create_solid_block_with_hole_o_t options;
    assert(GS_BODY_create_solid_block_with_hole_o_init(&options) == GS_ERROR_no_errors);
    assert(options.o_t_version == GS_O_T_VERSION && options.offset[0] == 0 && options.offset[1] == 0);
    options.o_t_version = 0;
    assert(GS_BODY_create_solid_block_with_hole(40, 25, 8, 4, NULL, &options, &body) == GS_ERROR_bad_option_version);
    options.o_t_version = GS_O_T_VERSION;
    options.offset[0] = 16;
    assert(GS_BODY_create_solid_block_with_hole(40, 25, 8, 4, NULL, &options, &body) == GS_ERROR_invalid_argument);
    options.offset[0] = NAN;
    assert(GS_BODY_create_solid_block_with_hole(40, 25, 8, 4, NULL, &options, &body) == GS_ERROR_invalid_argument);
    assert(body == -1);
    assert(GS_BODY_create_solid_block_with_hole(40, 25, 8, 4, NULL, NULL, &body) == GS_ERROR_no_errors);
    assert(body == 1); /* Rejections did not create bodies or consume tags. */
    const double world_bounds[3][2] = {{-20, 20}, {-12.5, 12.5}, {-4, 4}};
    check_body(body, world_bounds);
    GS_AXIS2_sf_t basis = {{10, 20, 30}, {1, 0, 0}, {0, 1, 0}};
    const double rotated_bounds[3][2] = {{6, 14}, {0, 40}, {17.5, 42.5}};
    options.offset[0] = 6;
    options.offset[1] = -3;
    assert(GS_BODY_create_solid_block_with_hole(40, 25, 8, 4, &basis, &options, &body) == GS_ERROR_no_errors);
    check_body(body, rotated_bounds);
    assert(GS_SESSION_stop() == GS_ERROR_no_errors);
    puts("C: all through-hole contracts passed");
    return 0;
}
