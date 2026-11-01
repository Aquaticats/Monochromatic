# Shared input content pages

The input ledger preserves original occurrences and complete immutable text.
Sharing text alone does not share the arrays that describe each snapshot.
The configured replay heap reaches those arrays through the manager's constructed-input reader,
the content store's `contents` map,
and each content reader's `chunks` closure.

The isolated sequence probe retains changing versions of a JSON catalog.
With 24 versions and 16,000 entries,
the original implementation retains 24,589,536 bytes.
Identical-input controls retain 1,091,016 and 1,029,504 bytes,
a spread of 61,512 bytes.
Every reconstructed version matches its original SHA-256 digest.
The changing-input case fails the eight-MiB additional-storage regression budget.
Receipt: `content-sequence-before.log` in the session's private verification directory.

`input-content-pieces.ts` owns detached chunks and immutable pages of at most 256 chunk references.
Complete versions retain page references.
Unchanged pages reuse their reference arrays as well as their text.
JSON arrays of original ordinals establish exact structural equality at both levels.
The existing UTF-8 digest remains metadata,
so distinct unpaired UTF-16 surrogates with identical UTF-8 digests remain distinct.

`input-content.ts` owns content-reader reuse and the original source/accessor identity check.
The reader reconstructs the original code units from pages without retaining its temporary serialization.
Source publications and occurrences remain independent of representation sharing.
Storage sharing grants no permission or instruction authority.

## Earlier insertions and page alignment

Fixed pages beginning at the first chunk duplicate reference arrays when new messages precede an unchanged catalog.
A private comparison retains the same live store and unique 16,000-entry catalog through collection.
Changing only a tail adds 83,688 measured bytes beyond unchanged controls;
growing a preceding message list adds 32,853,608 bytes and fails the eight-MiB guard.
Every original digest still matches,
so this is a storage failure rather than changed content.
The maintained regression also keeps the interner alive by calling it after measurement
and checking reuse of the original reader.

The partitioner now selects page ends from a rolling window of eight preceding chunk ordinals.
Pages contain at least 64 and at most 256 references,
except a final shorter page.
The fingerprint continues across page cuts so unchanged content can recover common boundaries after an insertion.
An absent anchor forces a bounded page;
a common anchor cannot produce arbitrarily short pages.

The polynomial fingerprint uses radix 31 and prime modulus 65,521.
The fixed window and bounded integer arithmetic give a linear pass with constant auxiliary space.
The fingerprint chooses boundaries only.
Exact complete ordinal sequences still establish page and content equality;
fingerprint or UTF-8 digest collisions cannot merge different code-unit sequences.
This does not promise constant storage for arbitrary unrelated input versions.
Maximum-length forced cuts also do not guarantee boundary reuse for every possible inserted-prefix sequence.
The unique-catalog regression establishes its tested workload;
adversarial anchor patterns and the configured host still require independent verification.

The maintained shifting-prefix guard fails at 32,937,656 additional measured bytes on the fixed-page source.
The content-aligned experiment passes that guard,
fresh builds,
full types and exact-content controls.
Its negative retained-heap delta is a measurement difference,
not a literal storage size or a quantified improvement.
The whole configured-host replay remains required to attribute its remaining growth to this change.
