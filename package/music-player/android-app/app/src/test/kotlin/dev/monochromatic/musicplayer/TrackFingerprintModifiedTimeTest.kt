// Cases of the modified-time read behind the track fingerprint: which providers are asked for the document column
// `last_modified`, what value each answer becomes, and how often a provider that refuses the column is asked again.
// Every authority, URI and time here is invented.
//
// What:     `package dev.monochromatic.musicplayer` places the cases beside `TrackFingerprint`.
// Why:      The read under test is `internal` to the app module, so the cases call it by its short name.
//
// In TS you'd write (pseudocode):
// ```ts
// // The file path is the module; the cases import the read from "./TrackFingerprint".
// ```
package dev.monochromatic.musicplayer

import android.content.ContentProvider
import android.content.ContentValues
import android.content.Context
import android.database.Cursor
import android.database.MatrixCursor
import android.net.Uri
import android.provider.DocumentsContract
import android.provider.MediaStore
import androidx.test.core.app.ApplicationProvider
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Robolectric
import org.robolectric.RobolectricTestRunner
import java.util.concurrent.CopyOnWriteArrayList

// What:     `private const val INVENTED_MODIFIED_MS: Long` is the time a document provider in these cases reports.
// Why:      A value far from zero shows that a read which succeeds hands the provider's own value back unchanged.
//
// In TS you'd write (pseudocode):
// ```ts
// const INVENTED_MODIFIED_MS = 1_700_000_000_123;
// ```
/** Modification time in milliseconds that the invented document provider reports for every row. */
private const val INVENTED_MODIFIED_MS: Long = 1_700_000_000_123L

// What:     `internal abstract class ColumnProvider` is a content provider that records the columns each query asked
//           for.
// Why:      Each case counts the queries that named the document column, which is the call the media provider logged as
//           `Invalid column last_modified` on the emulator.
//
// In TS you'd write (pseudocode):
// ```ts
// abstract class ColumnProvider { asked: string[][] = []; abstract answer(projection: string[]): Cursor | null; }
// ```
/** Records the projection of every query and lets each subclass decide the answer. */
internal abstract class ColumnProvider : ContentProvider() {
    /** Projections asked for so far, one entry per query, in call order. */
    val asked: MutableList<List<String>> = CopyOnWriteArrayList()

    /** Count of queries whose projection named the document modified-time column. */
    fun modifiedQueries(): Int = asked.count { columns -> DocumentsContract.Document.COLUMN_LAST_MODIFIED in columns }

    /** Answer of this provider to one projection: a cursor, null, or a thrown exception. */
    abstract fun answer(projection: List<String>): Cursor?

    /** Reports the provider as ready; it holds nothing to set up. */
    override fun onCreate(): Boolean = true

    /** Records [projection] and returns this provider's answer to it. */
    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?,
    ): Cursor? {
        /** Columns of this query, empty when the caller asked for every column. */
        val columns: List<String> = projection?.toList().orEmpty()
        asked += columns
        return answer(columns)
    }

    /** Has no type for any URI. */
    override fun getType(uri: Uri): String? = null

    /** Refuses inserts, which the read never makes. */
    override fun insert(uri: Uri, values: ContentValues?): Uri? = null

    /** Refuses deletes, which the read never makes. */
    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<out String>?): Int = 0

    /** Refuses updates, which the read never makes. */
    override fun update(
        uri: Uri,
        values: ContentValues?,
        selection: String?,
        selectionArgs: Array<out String>?,
    ): Int = 0
}

// What:     `internal class ColumnLackingProvider` throws on the document column, as the media provider does.
// Why:      The media provider of Android answers a projection with an unknown column with an
//           `IllegalArgumentException("Invalid column last_modified")`, which reaches the app across the binder.
//
// In TS you'd write (pseudocode):
// ```ts
// class ColumnLackingProvider extends ColumnProvider { answer() { throw new Error("Invalid column last_modified"); } }
// ```
/** Throws `IllegalArgumentException` for a query naming the document modified-time column, as the media store does. */
internal class ColumnLackingProvider : ColumnProvider() {
    /** Throws for the document column and answers any other projection with no cursor. */
    override fun answer(projection: List<String>): Cursor? {
        require(DocumentsContract.Document.COLUMN_LAST_MODIFIED !in projection) {
            "Invalid column ${DocumentsContract.Document.COLUMN_LAST_MODIFIED}"
        }
        return null
    }
}

// What:     `internal class DocumentProvider` answers the document column with one row holding the invented time.
// Why:      A document provider of the Storage Access Framework reports the column, and its value feeds the
//           fingerprint.
//
// In TS you'd write (pseudocode):
// ```ts
// class DocumentProvider extends ColumnProvider { answer(p) { return cursorOf(p, [INVENTED_MODIFIED_MS]); } }
// ```
/** Answers every projection with one row whose every column holds [INVENTED_MODIFIED_MS]. */
internal class DocumentProvider : ColumnProvider() {
    /** One row with the invented time in each asked column. */
    override fun answer(projection: List<String>): Cursor {
        /** Cursor whose columns are the asked ones. */
        val cursor = MatrixCursor(projection.toTypedArray())
        cursor.addRow(projection.map { INVENTED_MODIFIED_MS })
        return cursor
    }
}

// What:     `internal class RefusingProvider` throws `SecurityException` for every query.
// Why:      A refused row (a revoked grant) belongs to that one file, so the read must keep asking for the next file.
//
// In TS you'd write (pseudocode):
// ```ts
// class RefusingProvider extends ColumnProvider { answer() { throw new SecurityException("no grant"); } }
// ```
/** Throws `SecurityException` for every query, as a provider does for a document whose grant was revoked. */
internal class RefusingProvider : ColumnProvider() {
    /** Always refuses. */
    override fun answer(projection: List<String>): Cursor? = throw SecurityException("the invented grant was revoked")
}

// What:     `internal class UnknownRowProvider` throws `IllegalArgumentException` that does not name the column.
// Why:      An argument failure about one document must not stop the time reads of the provider's other documents,
//           whose fingerprints would otherwise change.
//
// In TS you'd write (pseudocode):
// ```ts
// class UnknownRowProvider extends ColumnProvider { answer() { throw new Error("Unknown document invented-7"); } }
// ```
/** Throws `IllegalArgumentException` naming an unknown document, not the column, for every query. */
internal class UnknownRowProvider : ColumnProvider() {
    /** Always refuses the one document. */
    override fun answer(projection: List<String>): Cursor? =
        throw IllegalArgumentException("Unknown document invented-7")
}

/**
 * Pins the modified-time read of [TrackFingerprint]: a media store URI is never asked for the document column, a
 * document provider's value is returned unchanged, a provider that refuses the column is asked once per process, and a
 * refused row does not stop the reads of other rows.
 */
@RunWith(RobolectricTestRunner::class)
class TrackFingerprintModifiedTimeTest {
    /** Application context whose content resolver reaches the invented providers. */
    private val context: Context = ApplicationProvider.getApplicationContext()

    /** URIs of [count] invented rows under [authority], numbered from one. */
    private fun rowsOf(authority: String, count: Int): List<Uri> =
        (1..count).map { row -> Uri.parse("content://$authority/invented/audio/$row") }

    /** The media store lacks the document column, so its rows read the unknown time without any query naming it. */
    @Test
    fun aMediaStoreRowIsNeverAskedForTheDocumentColumn() {
        /** Provider registered under the media store's own authority. */
        val provider: ColumnLackingProvider =
            Robolectric.setupContentProvider(ColumnLackingProvider::class.java, MediaStore.AUTHORITY)
        /** Times read for three invented media store rows. */
        val times: List<Long> = rowsOf(MediaStore.AUTHORITY, 3).map { uri ->
            TrackFingerprint.queryLastModifiedMs(context, uri)
        }
        assertEquals("every media store row reads the unknown time, as before", listOf(0L, 0L, 0L), times)
        assertEquals("no query names the column the media store lacks", 0, provider.modifiedQueries())
    }

    /** A document provider's value reaches the fingerprint unchanged, and each row is asked for its own value. */
    @Test
    fun aDocumentProvidersTimeIsReturnedUnchangedForEveryRow() {
        /** Provider of invented documents that reports the column. */
        val provider: DocumentProvider =
            Robolectric.setupContentProvider(DocumentProvider::class.java, "invented.documents")
        /** Times read for two invented documents. */
        val times: List<Long> = rowsOf("invented.documents", 2).map { uri ->
            TrackFingerprint.queryLastModifiedMs(context, uri)
        }
        assertEquals(listOf(INVENTED_MODIFIED_MS, INVENTED_MODIFIED_MS), times)
        assertEquals("each document is asked for its own time", 2, provider.modifiedQueries())
    }

    /** A provider that refuses the column is asked once, and every row after that reads the unknown time. */
    @Test
    fun aProviderThatRefusesTheColumnIsAskedOnce() {
        /** Provider under an invented authority that throws for the column, as the media store does. */
        val provider: ColumnLackingProvider =
            Robolectric.setupContentProvider(ColumnLackingProvider::class.java, "invented.lacking")
        /** Times read for five invented rows of that provider. */
        val times: List<Long> = rowsOf("invented.lacking", 5).map { uri ->
            TrackFingerprint.queryLastModifiedMs(context, uri)
        }
        assertEquals("every row reads the unknown time, as before", List(5) { 0L }, times)
        assertEquals("the refused column is asked for once, not once per row", 1, provider.modifiedQueries())
    }

    /** A refused row reads the unknown time without stopping the next row of the same provider from being asked. */
    @Test
    fun aRefusedRowDoesNotStopTheNextRowsRead() {
        /** Provider under an invented authority that refuses every row. */
        val provider: RefusingProvider =
            Robolectric.setupContentProvider(RefusingProvider::class.java, "invented.refusing")
        /** Times read for two invented rows of that provider. */
        val times: List<Long> = rowsOf("invented.refusing", 2).map { uri ->
            TrackFingerprint.queryLastModifiedMs(context, uri)
        }
        assertEquals(listOf(0L, 0L), times)
        assertEquals("a refusal belongs to its row, so the next row is asked too", 2, provider.modifiedQueries())
    }

    /** An argument failure that does not name the column belongs to its row, so the next row is asked too. */
    @Test
    fun anArgumentFailureAboutOneRowDoesNotStopTheNextRowsRead() {
        /** Provider under an invented authority that rejects every row as an unknown document. */
        val provider: UnknownRowProvider =
            Robolectric.setupContentProvider(UnknownRowProvider::class.java, "invented.unknown")
        /** Times read for two invented rows of that provider. */
        val times: List<Long> = rowsOf("invented.unknown", 2).map { uri ->
            TrackFingerprint.queryLastModifiedMs(context, uri)
        }
        assertEquals(listOf(0L, 0L), times)
        assertEquals("only a refusal naming the column is remembered", 2, provider.modifiedQueries())
    }
}
