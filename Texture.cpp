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

#include "Texture.hpp"

#include <GL/freeglut_std.h>
#include <GL/glext.h>
#include <memory>

#include "Image.hpp"

using namespace std;

Texture::Texture() {
	id = 0;
}

Texture::Texture(shared_ptr<Image> image) {
	unique_ptr<TextureDataBuffer> buffer = image->getPrimary();

	glGenTextures(1, &id);
	glBindTexture(GL_TEXTURE_2D, id);
	glTexStorage2D(GL_TEXTURE_2D, 1, GL_RGBA8, image->width(), image->height());
	glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, image->width(), image->height(), buffer->format, buffer->type, buffer->begin());
	glTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
	glTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
}

Texture::~Texture() {
	if (id != 0)
		glDeleteTextures(1, &id);
}

void Texture::render(int x __attribute__((unused)), int y __attribute__((unused)), int width __attribute__((unused)), int height __attribute__((unused))) {
	glBindTexture(GL_TEXTURE_2D, id);
}
