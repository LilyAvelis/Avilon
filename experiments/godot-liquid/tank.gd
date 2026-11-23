@tool
extends Node2D

const CELLS := 92
const WIDTH := 636.0
const HEIGHT := 262.0
const DX := WIDTH / CELLS
const LEFT := -WIDTH * 0.5
const TOP := -HEIGHT * 0.5
const BOTTOM := HEIGHT * 0.5
const MIN_DEPTH := 3.0
const MAX_DEPTH := HEIGHT - 5.0

@export_range(0.39, 0.61, 0.01) var fill_ratio := 0.49
@export_range(150.0, 320.0, 1.0) var wave_speed := 250.0
@export_range(0.2, 3.0, 0.05) var damping := 0.9

var target_degrees := 0.0
var _depth := PackedFloat32Array()
var _flux := PackedFloat32Array()
var _water_indices := PackedInt32Array()


func _ready() -> void:
	# Each pair of neighboring surface samples always spans the same two triangles.
	for index in range(CELLS + 1):
		var top := index * 2
		_water_indices.append_array([top, top + 2, top + 1, top + 1, top + 2, top + 3])
	reset()


func reset() -> void:
	target_degrees = 0.0
	rotation = 0.0
	_depth.resize(CELLS)
	_depth.fill(HEIGHT * fill_ratio)
	_flux.resize(CELLS + 1)
	_flux.fill(0.0)
	queue_redraw()


func _physics_process(delta: float) -> void:
	if Engine.is_editor_hint():
		if not is_equal_approx(_depth[0], HEIGHT * fill_ratio):
			reset()
		return
	rotation = lerpf(rotation, deg_to_rad(target_degrees), 1.0 - exp(-6.5 * delta))
	for substep in range(3):
		_step(delta / 3.0)
	queue_redraw()


func _step(dt: float) -> void:
	var slope := tan(rotation)
	var drag := exp(-damping * dt)
	for face in range(1, CELLS):
		var left_depth := _depth[face - 1]
		var right_depth := _depth[face]
		var pressure_slope := (right_depth - left_depth) / DX - slope
		var next := (_flux[face] - wave_speed * wave_speed * pressure_slope * dt) * drag
		# Each cell can exchange with two neighbors, so each face uses at most half its capacity.
		var available := minf(left_depth - MIN_DEPTH, MAX_DEPTH - right_depth)
		var reverse := minf(right_depth - MIN_DEPTH, MAX_DEPTH - left_depth)
		var capacity := 0.49 * DX / dt
		_flux[face] = clampf(next, -maxf(0.0, reverse) * capacity, maxf(0.0, available) * capacity)
	for index in range(CELLS):
		_depth[index] += (_flux[index] - _flux[index + 1]) * dt / DX


func stir_at(world_position: Vector2, strength: float) -> void:
	var local := to_local(world_position)
	if absf(local.x) >= WIDTH * 0.5 or local.y > BOTTOM:
		return
	var column := clampi(int((local.x - LEFT) / DX), 0, CELLS - 1)
	if local.y < BOTTOM - _depth[column] - 28.0:
		return
	for face in range(1, CELLS):
		var distance := (LEFT + DX * face - local.x) / 39.0
		_flux[face] += 12500.0 * strength * distance * exp(-0.5 * distance * distance)


func _draw() -> void:
	if _depth.is_empty():
		return
	var glass := Rect2(LEFT, TOP, WIDTH, HEIGHT)
	var shadow := glass.grow(14.0)
	shadow.position.y += 16.0
	draw_rect(shadow, Color("#07131c"))
	draw_rect(glass.grow(11.0), Color("#374b55"))
	draw_rect(glass.grow(8.0), Color("#132833"))
	draw_rect(glass, Color("#102733"))
	for index in range(1, 12):
		var x := LEFT + WIDTH * float(index) / 12.0
		draw_line(Vector2(x, TOP + 1.0), Vector2(x, BOTTOM), Color(0.61, 0.75, 0.79, 0.08))
	for index in range(1, 6):
		var y := TOP + HEIGHT * float(index) / 6.0
		draw_line(Vector2(LEFT + 1.0, y), Vector2(-LEFT - 1.0, y), Color(0.61, 0.75, 0.79, 0.10))

	var surface := PackedVector2Array()
	surface.append(Vector2(LEFT, BOTTOM - _depth[0]))
	for column in range(CELLS):
		surface.append(Vector2(LEFT + (column + 0.5) * DX, BOTTOM - _depth[column]))
	surface.append(Vector2(-LEFT, BOTTOM - _depth[CELLS - 1]))
	var pool := PackedVector2Array()
	var pigment := PackedColorArray()
	for point in surface:
		pool.append(point)
		pool.append(Vector2(point.x, BOTTOM))
		var rise := (BOTTOM - point.y) / HEIGHT
		pigment.append(Color("#3f9d9c").lerp(Color("#47b3aa"), rise))
		pigment.append(Color("#153e63"))
	RenderingServer.canvas_item_add_triangle_array(get_canvas_item(), _water_indices, pool, pigment)
	for layer in [0.14, 0.29, 0.50, 0.72]:
		var contour := PackedVector2Array()
		for point in surface:
			contour.append(Vector2(point.x, lerpf(point.y, BOTTOM, layer)))
		draw_polyline(contour, Color(0.71, 0.95, 0.90, 0.15), 1.0, true)
	draw_polyline(surface, Color(0.53, 0.99, 0.87, 0.15), 10.0, true)
	draw_polyline(surface, Color("#c4f6df"), 2.0, true)

	for side in [-1.0, 1.0]:
		var x: float = side * (WIDTH * 0.5 - 5.0)
		draw_line(Vector2(x, TOP + 9.0), Vector2(x, BOTTOM - 9.0), Color(0.73, 0.90, 0.89, 0.34), 1.25, true)
	draw_rect(glass, Color("#83a6aa"), false, 1.0)
	draw_line(Vector2(LEFT - 8.0, TOP - 7.0), Vector2(-LEFT + 8.0, TOP - 7.0), Color("#69838a"), 1.0, true)
	for x in [LEFT - 5.0, -LEFT + 5.0]:
		for y in [TOP - 5.0, BOTTOM + 5.0]:
			draw_circle(Vector2(x, y), 2.5, Color("#a78e6d"), true, -1.0, true)
