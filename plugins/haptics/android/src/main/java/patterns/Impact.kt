// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.haptics.patterns

val ImpactPatternLight = Pattern(
    longArrayOf(0, 50),
    intArrayOf(0, 30),
    longArrayOf(0, 20)
)

val ImpactPatternMedium = Pattern(
    longArrayOf(0, 43),
    intArrayOf(0, 50),
    longArrayOf(0, 43)
)

val ImpactPatternHeavy = Pattern(
    longArrayOf(0, 60),
    intArrayOf(0, 70),
    longArrayOf(0, 61)
)

// longer and weaker than Light, a muted impact
val ImpactPatternSoft = Pattern(
    longArrayOf(0, 70),
    intArrayOf(0, 20),
    longArrayOf(0, 30)
)

// shorter and stronger than Medium, a sharp impact
val ImpactPatternRigid = Pattern(
    longArrayOf(0, 25),
    intArrayOf(0, 90),
    longArrayOf(0, 15)
)