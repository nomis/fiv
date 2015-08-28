################################################################################
# Automatically-generated file. Do not edit!
################################################################################

# Add inputs and outputs from these tool invocations to the build variables 
CPP_SRCS += \
../Fiv.cpp \
../Image.cpp \
../main.cpp 

OBJS += \
./Fiv.o \
./Image.o \
./main.o 

CPP_DEPS += \
./Fiv.d \
./Image.d \
./main.d 


# Each subdirectory must supply rules for building sources it contributes
%.o: ../%.cpp
	@echo 'Building file: $<'
	@echo 'Invoking: GCC C++ Compiler'
	${CXX} $(CXXFLAGS) -std=c++14 -D_FILE_OFFSET_BITS=64 -O2 -g -Wall -Wextra -Werror -c -fmessage-length=0 -Wshadow -MMD -MP -MF"$(@:%.o=%.d)" -MT"$(@:%.o=%.d)" -o "$@" "$<"
	@echo 'Finished building: $<'
	@echo ' '


