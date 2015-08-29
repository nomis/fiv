################################################################################
# Automatically-generated file. Do not edit!
################################################################################

# Add inputs and outputs from these tool invocations to the build variables 
CPP_SRCS += \
../Codec.cpp \
../Fiv.cpp \
../Image.cpp \
../JpegCodec.cpp \
../Magic.cpp \
../main.cpp 

OBJS += \
./Codec.o \
./Fiv.o \
./Image.o \
./JpegCodec.o \
./Magic.o \
./main.o 

CPP_DEPS += \
./Codec.d \
./Fiv.d \
./Image.d \
./JpegCodec.d \
./Magic.d \
./main.d 


# Each subdirectory must supply rules for building sources it contributes
%.o: ../%.cpp
	@echo 'Building file: $<'
	@echo 'Invoking: GCC C++ Compiler'
	${CXX} $(CXXFLAGS) -std=c++14 -D_FILE_OFFSET_BITS=64 -O2 -g -Wall -Wextra -Werror -c -fmessage-length=0 -Wshadow -MMD -MP -MF"$(@:%.o=%.d)" -MT"$(@:%.o=%.d)" -o "$@" "$<"
	@echo 'Finished building: $<'
	@echo ' '


