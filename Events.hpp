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

#ifndef fiv__EVENTS_HPP_
#define fiv__EVENTS_HPP_

#include <memory>

class Events:  public std::enable_shared_from_this<Events> {
public:
	Events();
	virtual ~Events();
	virtual void addImage();
	virtual void loadedCurrent();
};

#endif /* fiv__EVENTS_HPP_ */
