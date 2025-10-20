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

# Output library and CLI
TARGET = libevoting_crypto.so
DKG_CLI = dkg_cli

# Default target
all: $(TARGET) $(DKG_CLI)

# Build shared library
$(TARGET): $(OBJECTS)
	$(CXX) $(LDFLAGS) -o $@ $^ $(LIBS)

# Build object files
%.o: %.cpp
	$(CXX) $(CXXFLAGS) -c $< -o $@

# Build DKG CLI executable
$(DKG_CLI): dkg_cli.cpp $(OBJECTS)
	$(CXX) $(CXXFLAGS) -o $@ $< $(OBJECTS) $(LIBS)

# Clean build files
clean:
	rm -f $(OBJECTS) $(TARGET) $(DKG_CLI)

# Install library (optional)
install: $(TARGET)
	cp $(TARGET) /usr/local/lib/
	ldconfig

.PHONY: all clean install
