"""
Convert Cornell Box scene.json → binary glTF 2.0 (GLB).

Reads surface geometry, materials, and metadata from scene.json
(the single source of truth for official Cornell Box data).
Output: cornell_box.glb

Usage: python convert_cornell_gltf.py
"""

import json, struct


def main():
    with open("models/cornell_box/scene.json") as f:
        scene = json.load(f)

    materials = scene["materials"]
    surfaces = scene["surfaces"]

    # ── Coordinate transform ───────────────────────────────────────────
    # Official mm → renderer world space.
    # x = -(X - 278)  (handedness: official right=-X, ours right=+X)
    # y = Y           (mm as-is)
    # z = Z           (mm as-is)
    def to_our(v):
        return (-(v[0] - 278.0), v[1], v[2])

    # ── Build vertex/index buffers ────────────────────────────────────
    all_verts = []       # flat [x, y, z, ...]
    vertex_map = {}      # (x,y,z) → 0-based index
    mat_groups = {}      # material_name → list of triangle indices

    for s in surfaces:
        mat_name = s["material"]
        quads = s["vertices_official_mm"]
        for i in range(0, len(quads), 4):
            vv = [to_our(quads[i + j]) for j in range(4)]
            for v in vv:
                key = (round(v[0], 6), round(v[1], 6), round(v[2], 6))
                if key not in vertex_map:
                    vertex_map[key] = len(all_verts) // 3
                    all_verts.extend(key)
            i0 = vertex_map[(round(vv[0][0], 6), round(vv[0][1], 6), round(vv[0][2], 6))]
            i1 = vertex_map[(round(vv[1][0], 6), round(vv[1][1], 6), round(vv[1][2], 6))]
            i2 = vertex_map[(round(vv[2][0], 6), round(vv[2][1], 6), round(vv[2][2], 6))]
            i3 = vertex_map[(round(vv[3][0], 6), round(vv[3][1], 6), round(vv[3][2], 6))]
            tris = mat_groups.setdefault(mat_name, [])
            tris.extend([i0, i1, i2, i0, i2, i3])

    # ── Build GLB ─────────────────────────────────────────────────────
    buffer_data = bytearray()

    def pad4(data):
        while len(data) % 4:
            data.append(0)
        return data

    verts_bytes = struct.pack(f"<{len(all_verts)}f", *all_verts)
    verts_off = len(buffer_data)
    buffer_data.extend(verts_bytes)
    buffer_data = pad4(buffer_data)

    gltf = {
        "asset": {"version": "2.0", "generator": "convert_cornell_gltf.py (scene.json)"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": []}],
        "materials": [],
        "accessors": [
            {
                "bufferView": 0, "componentType": 5126,
                "count": len(all_verts) // 3, "type": "VEC3",
                "min": [min(all_verts[i::3]) for i in range(3)],
                "max": [max(all_verts[i::3]) for i in range(3)],
            }
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": verts_off, "byteLength": len(verts_bytes)},
        ],
        "buffers": [{"byteLength": 0}],
    }

    for mat_name in ("red", "green", "white"):
        tris = mat_groups.get(mat_name, [])
        if not tris:
            continue

        max_idx = max(tris)
        fmt = "<I" if max_idx > 65535 else "<H"
        comp_type = 5125 if max_idx > 65535 else 5123
        idx_bytes = struct.pack(f"<{len(tris)}{fmt[1]}", *tris)
        idx_off = len(buffer_data)
        buffer_data.extend(idx_bytes)
        buffer_data = pad4(buffer_data)

        bv_idx = len(gltf["bufferViews"])
        gltf["bufferViews"].append({
            "buffer": 0, "byteOffset": idx_off, "byteLength": len(idx_bytes),
            "target": 34963,
        })

        mat = materials[mat_name]
        mat_idx = len(gltf["materials"])
        gltf["materials"].append({
            "name": mat_name,
            "doubleSided": mat["double_sided"],
            "pbrMetallicRoughness": {
                "baseColorFactor": [mat["albedo"]["r"], mat["albedo"]["g"], mat["albedo"]["b"], 1.0],
                "roughnessFactor": mat["roughness"],
                "metallicFactor": 0.0,
            },
        })
        acc_idx = len(gltf["accessors"])
        gltf["accessors"].append({
            "bufferView": bv_idx, "componentType": comp_type,
            "count": len(tris), "type": "SCALAR",
        })
        gltf["meshes"][0]["primitives"].append({
            "attributes": {"POSITION": 0},
            "indices": acc_idx,
            "material": mat_idx,
        })

    gltf["buffers"][0]["byteLength"] = len(buffer_data)
    buffer_data = pad4(buffer_data)

    json_bytes = json.dumps(gltf, separators=(",", ":")).encode("utf-8")
    while len(json_bytes) % 4:
        json_bytes += b" "
    json_bytes = pad4(bytearray(json_bytes))

    total = 12 + 8 + len(json_bytes) + 8 + len(buffer_data)
    header = struct.pack("<4sII", b"glTF", 2, total)
    json_chunk = struct.pack("<II", len(json_bytes), 0x4E4F534A) + json_bytes
    bin_chunk = struct.pack("<II", len(buffer_data), 0x004E4942) + bytes(buffer_data)

    with open("models/cornell_box/cornell_box.glb", "wb") as f:
        f.write(header + json_chunk + bin_chunk)

    print(f"GLB written: {len(all_verts)//3} vertices, {len(gltf['meshes'][0]['primitives'])} primitives")


if __name__ == "__main__":
    main()
