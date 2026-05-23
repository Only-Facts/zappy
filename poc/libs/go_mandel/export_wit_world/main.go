package export_wit_world

import (
	"wit_component/local_zappy_command"
	"wit_component/local_zappy_graphic"
	"wit_component/local_zappy_host_api"
	"wit_component/local_zappy_input"
	"wit_component/local_zappy_system"
)

var shouldDrawMandel bool = false

func GetCommands() []local_zappy_command.CommandDesc {
	return []local_zappy_command.CommandDesc{
		{
			Module:  "gobot",
			Name:    "go_mandel",
			Options: "",
			Help:    "Mandelbrot propulsé par le compilateur Go standard !",
		},
	}
}

func RunCommand(cmd string, args []string) local_zappy_command.ResponseCommand {
	if cmd == "go_mandel" {
		local_zappy_host_api.HostLog("[Gobot] Commande go_mandel exécutée avec succès !")

		shouldDrawMandel = true

		return local_zappy_command.MakeResponseCommandOk()
	}
	return local_zappy_command.MakeResponseCommandUnknown()
}

func UpdateModule(time float32, dt float32, w float32, h float32) []local_zappy_graphic.RenderCommand {
	if !shouldDrawMandel {
		return nil
	}

	cols := 80
	rows := 60
	blockW := w / float32(cols)
	blockH := h / float32(rows)

	var commands []local_zappy_graphic.RenderCommand
	maxIteration := 32

	for py := 0; py < rows; py++ {
		for px := 0; px < cols; px++ {
			x0 := (float32(px)/float32(cols))*3.5 - 2.5
			y0 := (float32(py)/float32(rows))*2.0 - 1.0

			var x, y float32 = 0.0, 0.0
			var iteration int = 0

			for x*x+y*y <= 4.0 && iteration < maxIteration {
				xtemp := x*x - y*y + x0
				y = 2.0*x*y + y0
				x = xtemp
				iteration++
			}

			var r, g, b uint8
			if iteration < maxIteration {
				r = uint8((iteration * 8) % 256)
				g = uint8((iteration * 4) % 256)
				b = uint8((iteration * 16) % 256)
			}

			rect := local_zappy_graphic.RectCmd{
				X:        float32(px) * blockW,
				Y:        float32(py) * blockH,
				W:        blockW,
				H:        blockH,
				Color:    local_zappy_graphic.Color{R: r, G: g, B: b, A: 255},
				Rotation: 0.0,
			}

			commands = append(commands, local_zappy_graphic.MakeRenderCommandRect(rect))
		}
	}

	return commands
}

func HandleInput(state local_zappy_input.InputState)      {}
func AcceptLog(segments []local_zappy_system.TextSegment) {}
func Serialize() []uint8                                  { return nil }
func Deserialize(state []uint8)                           {}
func HandleEvent(eventName string, payload string)        {}
