################################################################################
# Automatically-generated file. Do not edit!
################################################################################

# Add inputs and outputs from these tool invocations to the build variables 
CPP_SRCS += \
../Application.cpp \
../Codec.cpp \
../Codecs.cpp \
../DataBuffer.cpp \
../FileDataBuffer.cpp \
../Fiv.cpp \
../FivImages.cpp \
../Image.cpp \
../ImageDrawable.cpp \
../JpegCodec.cpp \
../Magic.cpp \
../MainWindow.cpp \
../MemoryDataBuffer.cpp \
../main.cpp 

OBJS += \
./Application.o \
./Codec.o \
./Codecs.o \
./DataBuffer.o \
./FileDataBuffer.o \
./Fiv.o \
./FivImages.o \
./Image.o \
./ImageDrawable.o \
./JpegCodec.o \
./Magic.o \
./MainWindow.o \
./MemoryDataBuffer.o \
./main.o 

CPP_DEPS += \
./Application.d \
./Codec.d \
./Codecs.d \
./DataBuffer.d \
./FileDataBuffer.d \
./Fiv.d \
./FivImages.d \
./Image.d \
./ImageDrawable.d \
./JpegCodec.d \
./Magic.d \
./MainWindow.d \
./MemoryDataBuffer.d \
./main.d 


# Each subdirectory must supply rules for building sources it contributes
%.o: ../%.cpp
	@echo 'Building file: $<'
	@echo 'Invoking: GCC C++ Compiler'
	${CXX} $(shell pkg-config --cflags cairomm-1.0) $(shell pkg-config --cflags exiv2) $(shell pkg-config --cflags gtkmm-3.0) $(CXXFLAGS) -std=c++14 -D_FILE_OFFSET_BITS=64 -DGL_GLEXT_PROTOTYPES -O2 -g -Wall -Wextra -Werror -c -fmessage-length=0 -Wshadow -MMD -MP -MF"$(@:%.o=%.d)" -MT"$(@:%.o=%.d)" -o "$@" "$<"
	@echo 'Finished building: $<'
	@echo ' '


