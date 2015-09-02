/*
 Copyright 2015  Simon Arlott

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#ifndef fiv__TEXTURE_HPP_
#define fiv__TEXTURE_HPP_

#include <GL/freeglut_std.h>
#include <GL/glext.h>
#include <memory>

class Image;

class Texture {
public:
	Texture();
	Texture(std::shared_ptr<Image> image);
	virtual ~Texture();
	void render(int x, int y, int width, int height);

private:
	Texture(const Texture&) = delete;

	GLuint id;
};

#endif /* fiv__TEXTURE_HPP_ */
