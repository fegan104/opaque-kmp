package io.github.kotlin.fibonacci

import dev.oxikt.opaquekmp.FibonacciGenerator

fun generateFibi(generator: FibonacciGenerator = FibonacciGenerator(firstElement, secondElement)) = sequence {
    while (true) {
        val value = generator.next() ?: break
        yield(value)
    }
}

expect val firstElement: Int
expect val secondElement: Int
