################################################################################
# Automatically-generated file. Do not edit!
################################################################################

# Add inputs and outputs from these tool invocations to the build variables 
CPP_SRCS += \
../Codec.cpp \
../DataBuffer.cpp \
../FileDataBuffer.cpp \
../Fiv.cpp \
../Image.cpp \
../JpegCodec.cpp \
../Magic.cpp \
../MainWindow.cpp \
../MemoryDataBuffer.cpp \
../Window.cpp \
../main.cpp 

OBJS += \
./Codec.o \
./DataBuffer.o \
./FileDataBuffer.o \
./Fiv.o \
./Image.o \
./JpegCodec.o \
./Magic.o \
./MainWindow.o \
./MemoryDataBuffer.o \
./Window.o \
./main.o 

CPP_DEPS += \
./Codec.d \
./DataBuffer.d \
./FileDataBuffer.d \
./Fiv.d \
./Image.d \
./JpegCodec.d \
./Magic.d \
./MainWindow.d \
./MemoryDataBuffer.d \
./Window.d \
./main.d 


# Each subdirectory must supply rules for building sources it contributes
%.o: ../%.cpp
	@echo 'Building file: $<'
	@echo 'Invoking: GCC C++ Compiler'
	${CXX} $(CXXFLAGS) -std=c++14 -D_FILE_OFFSET_BITS=64 -O2 -g -Wall -Wextra -Werror -c -fmessage-length=0 -Wshadow -MMD -MP -MF"$(@:%.o=%.d)" -MT"$(@:%.o=%.d)" -o "$@" "$<"
	@echo 'Finished building: $<'
	@echo ' '


