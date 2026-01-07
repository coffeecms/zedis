<?php
require 'vendor/autoload.php';

$client = new Predis\Client();

echo "--- 11 Transactions ---\n";

$key = "php_txn";
$client->set($key, 10);

$responses = $client->transaction(function ($tx) use ($key) {
    $tx->incr($key);
    $tx->incrby($key, 5);
});

echo "Transaction Results: " . json_encode($responses) . "\n";
echo "Final Value: " . $client->get($key) . "\n";
?>