extends Node2D

@onready var tank: Node2D = $Tank
@onready var tilt: HSlider = $Interface/Tilt
@onready var angle: Label = $Interface/Angle


func _ready() -> void:
	tilt.value_changed.connect(_set_tilt)
	$Interface/Reset.pressed.connect(_reset)
	tank.stir_at(tank.position + Vector2(-90.0, 0.0), 0.26)


func _set_tilt(value: float) -> void:
	tank.target_degrees = value
	angle.text = "%+d°" % int(value)


func _reset() -> void:
	tilt.value = 0.0
	tank.reset()


func _unhandled_key_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed:
		if event.physical_keycode == KEY_R or event.keycode == KEY_R:
			_reset()
			get_viewport().set_input_as_handled()
			return
		match event.keycode:
			KEY_LEFT:
				tilt.value -= 1.0
			KEY_RIGHT:
				tilt.value += 1.0
			_:
				return
		get_viewport().set_input_as_handled()


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		tank.stir_at(get_global_mouse_position(), 1.0)
	elif event is InputEventMouseMotion and event.button_mask & MOUSE_BUTTON_MASK_LEFT:
		var strength := minf(event.relative.length() * 0.013, 0.35)
		tank.stir_at(get_global_mouse_position(), strength)
