package fit.eveship.dogma

import java.nio.file.Files
import java.nio.file.StandardCopyOption

internal object Native {
    private const val LIBRARY = "esf_dogma_engine_jni"

    init {
        val name = System.mapLibraryName(LIBRARY)
        val resource = Native::class.java.getResourceAsStream("natives/${platform()}/$name")
        if (resource == null) {
            System.loadLibrary(LIBRARY)
        } else {
            val file = Files.createTempFile("esf-dogma-engine", name)
            file.toFile().deleteOnExit()
            resource.use { Files.copy(it, file, StandardCopyOption.REPLACE_EXISTING) }
            System.load(file.toAbsolutePath().toString())
        }
    }

    private fun platform(): String {
        val os = System.getProperty("os.name").lowercase()
        val arch = System.getProperty("os.arch").lowercase()
        val osName = when {
            os.startsWith("linux") -> "linux"
            os.startsWith("mac") -> "macos"
            os.startsWith("windows") -> "windows"
            else -> os
        }
        val archName = when (arch) {
            "amd64", "x86_64" -> "x86_64"
            "aarch64", "arm64" -> "aarch64"
            else -> arch
        }
        return "$osName-$archName"
    }

    @JvmStatic external fun loadSde(bytes: ByteArray): Int
    @JvmStatic external fun loadNames(bytes: ByteArray): Int
    @JvmStatic external fun loadEft(eft: String): String
    @JvmStatic external fun saveEft(fit: String): String
    @JvmStatic external fun loadEsiFitting(fitting: String): String
    @JvmStatic external fun saveEsiFitting(fit: String): String
    @JvmStatic external fun postLoad(fit: String): String
    @JvmStatic external fun loadKillmail(killmail: String): String
    @JvmStatic external fun loadDna(dna: String): String
    @JvmStatic external fun loadLink(version: String, payload: String): String
    @JvmStatic external fun loadEsf(text: String): String
    @JvmStatic external fun loadEsfLink(link: String): String
    @JvmStatic external fun saveEsf(fit: String): String
    @JvmStatic external fun saveEsfLink(fit: String): String
    @JvmStatic external fun calculate(fit: String, options: String): String
    @JvmStatic external fun beacon(typeId: Int): String
}
