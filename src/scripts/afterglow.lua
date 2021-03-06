-- This file is part of Eruption.

-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.

-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.

-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

-- global state variables --
color_map = {}
color_map_glow = {}
ticks = 0

-- event handler functions --
function on_startup(config)
	for i = 0, get_num_keys() do
		color_map[i] = 0x000000000
	end
end

function on_key_down(key_index)
   color_map[key_index] = rgba_to_color(255, 255, 255, 255) -- color_afterglow
end

function on_tick(delta)
    ticks = ticks + delta + 1

    -- calculate afterglow effect for pressed keys
    if ticks % afterglow_step == 0 then
				local num_keys = get_num_keys()

        for i = 0, num_keys do
					r, g, b, alpha = color_to_rgba(color_map[i])
					if alpha > 0 then
						color_map[i] = rgba_to_color(r, g, b, max(alpha - 1, 0))
					else
						color_map[i] = 0x000000000
					end
				end

			submit_color_map(color_map)
    end
end
