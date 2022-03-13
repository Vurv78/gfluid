require("fluid")

---@class Particle
---@field imass number
---@field phase number
---@field velocity Vector
---@field position Vector

---@class Shape
---@field pos Vector
---@field rot table
---@field kind integer

---@type table<number, Particle>
local Particles = {}
local Boxes = {}
timer.Create("gfluid_sync", 1 / 20, 0, function()
	Particles = flex.getParticles()
	Boxes = flex.getBoxes()
end)

local NoAng = Angle()
local Size = Vector(5 ,5, 5)

local White = Color(255, 255, 255)
local Red = Color(255, 0, 0)
hook.Add("PostDrawTranslucentRenderables", "gfluid_render", function()
	render.SetColorMaterial()
	for _, particle in ipairs(Particles) do
		local pos = particle.position
		render.DrawSphere( particle.position, 20, 50, 50, White )
	end

	for _, box in ipairs(Boxes) do
		render.DrawBox( box.pos, NoAng, Size, Size, Red )
	end
end)
