# Compiler settings
CXX = g++
CXXFLAGS = -std=c++17 -O2 -fPIC -Wall
LDFLAGS = -shared
LIBS = -lgmp -lpbc -lcrypto

# Source files
SOURCES = setup.cpp ffi_wrapper.cpp keygen.cpp didgen.cpp prepareblindsign.cpp blindsign.cpp \
          unblindsign.cpp aggregate.cpp provecredential.cpp pairinginverify.cpp \
          checkkorverify.cpp kor.cpp

# Object files
OBJECTS = $(SOURCES:.cpp=.o)

# Output library
TARGET = libevoting_crypto.so

# Default target
all: $(TARGET)

# Build shared library
$(TARGET): $(OBJECTS)
	$(CXX) $(LDFLAGS) -o $@ $^ $(LIBS)

# Build object files
%.o: %.cpp
	$(CXX) $(CXXFLAGS) -c $< -o $@

# Clean build files
clean:
	rm -f $(OBJECTS) $(TARGET)

# Install library (optional)
install: $(TARGET)
	cp $(TARGET) /usr/local/lib/
	ldconfig

.PHONY: all clean install
