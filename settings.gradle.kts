pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "ind-os"

include(":packages:IndIpc")
project(":packages:IndIpc").projectDir = file("packages/IndIpc")

include(":packages:IndNeuralCore")
project(":packages:IndNeuralCore").projectDir = file("packages/IndNeuralCore")

include(":packages:IndZeroTouch")
project(":packages:IndZeroTouch").projectDir = file("packages/IndZeroTouch")

include(":packages:IndSurakshaShield")
project(":packages:IndSurakshaShield").projectDir = file("packages/IndSurakshaShield")

include(":packages:IndLauncher")
project(":packages:IndLauncher").projectDir = file("packages/IndLauncher")
