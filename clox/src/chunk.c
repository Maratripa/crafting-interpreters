#include <stdlib.h>

#include "chunk.h"
#include "memory.h"
#include "value.h"
#include "vm.h"

void initChunk(Chunk* chunk) {
  chunk->count = 0;
  chunk->capacity = 0;
  chunk->code = NULL;

  chunk->lineCount = 0;
  chunk->lineCapacity = 0;
  chunk->lines = NULL;

  initValueArray(&chunk->constants);
}

void freeChunk(Chunk* chunk) {
  FREE_ARRAY(uint8_t, chunk->code, chunk->capacity);
  FREE_ARRAY(int, chunk->lines, chunk->lineCapacity);
  freeValueArray(&chunk->constants);
  initChunk(chunk);
}

void writeChunk(Chunk* chunk, uint8_t byte, int line) {
  if (chunk->capacity < chunk->count + 1) {
    int oldCapacity = chunk->capacity;
    chunk->capacity = GROW_CAPACITY(oldCapacity);
    chunk->code = GROW_ARRAY(uint8_t, chunk->code, oldCapacity, chunk->capacity);
  }

  if (chunk->lineCount > 0 && chunk->lines[chunk->lineCount - 2] == line) {
    chunk->lines[chunk->lineCount - 1]++;
  } else {
    if (chunk->lineCapacity < chunk->lineCount + 2) {
      int oldCapacity = chunk->lineCapacity;
      chunk->lineCapacity = GROW_CAPACITY(oldCapacity);
      chunk->lines = GROW_ARRAY(int, chunk->lines, oldCapacity, chunk->lineCapacity);
    }

    chunk->lines[chunk->lineCount] = line;
    chunk->lines[chunk->lineCount + 1] = 1;

    chunk->lineCount = chunk->lineCount + 2;
  }

  chunk->code[chunk->count] = byte;
  chunk->count++;
}

void writeConstant(Chunk* chunk, Value value, int line) {
  int index = addConstant(chunk, value);
  if (index < 256) {
    writeChunk(chunk, OP_CONSTANT, line);
    writeChunk(chunk, (uint8_t)index, line);
  } else {
    writeChunk(chunk, OP_CONSTANT_LONG, line);
    writeChunk(chunk, (uint8_t)(index & 0xff), line);
    writeChunk(chunk, (uint8_t)((index >> 8) & 0xff), line);
    writeChunk(chunk, (uint8_t)((index >> 16) & 0xff), line);
  }
}

int addConstant(Chunk* chunk, Value value) {
  push(value);
  writeValueArray(&chunk->constants, value);
  pop();
  return chunk->constants.count - 1;
}

/*
  Lines are stored in a dynamic array in the following way:

  {line, count, line, count, line, count}
*/
int getLine(Chunk* chunk, int index) {
  int sum = 0;
  for (int i = 1; i < chunk->lineCount; i = i + 2) {
    sum = sum + chunk->lines[i];

    if (index < sum) return chunk->lines[i - 1];
  }

  return -1;
}

