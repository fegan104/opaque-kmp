package io.github.kotlin.fibonacci

import com.frankegan.opaque_kmp.FibonacciGenerator

fun generateFibi(generator: FibonacciGenerator = FibonacciGenerator(firstElement, secondElement)) = sequence {
    while (true) {
        val value = generator.next() ?: break
        yield(value)
    }
}

expect val firstElement: Int
expect val secondElement: Int
