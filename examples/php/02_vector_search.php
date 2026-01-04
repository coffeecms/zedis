<?php
require 'vendor/autoload.php';

$client = new Predis\Client([
    'host' => '127.0.0.1',
    'port' => 6379,
]);

echo "--- 02 AI Vector Search (Auto-Embedding) ---\n";

// 1. Ingest
echo "Ingesting books...\n";
$books = [
    "book:1" => "Introduction to Algorithms by Cormen",
    "book:2" => "Clean Code by Robert Martin",
    "book:3" => "The Pragmatic Programmer",
    "book:4" => "Harry Potter and the Sorcerer's Stone"
];

foreach ($books as $key => $title) {
    // VADD.TEXT <key> <content>
    $client->executeRaw(['VADD.TEXT', $key, $title]);
}

// 2. Search
$query = "coding best practices";
echo "\nSearching for: '$query'\n";

// VSEARCH.TEXT <index> <query> <limit>
$results = $client->executeRaw(['VSEARCH.TEXT', 'book', $query, '2']);
print_r($results);
// Expected: book:2 and book:3
?>