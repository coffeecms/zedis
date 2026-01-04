<?php
require 'vendor/autoload.php';

$client = new Predis\Client([
    'host' => '127.0.0.1',
    'port' => 6379,
]);

echo "--- 07 Complex Structures ---\n";

// List
$client->lpush('notifications', 'alert:1');
$client->lpush('notifications', 'alert:2');
$msg = $client->rpop('notifications');
echo "Popped: $msg\n";

// Set
$client->sadd('online_users', 'user:1', 'user:2');
$count = $client->scard('online_users');
echo "Online users: $count\n";

// Hash
$client->hmset('vehicle:1', [
    'model' => 'Tesla',
    'year' => '2024'
]);
$model = $client->hget('vehicle:1', 'model');
echo "Vehicle Model: $model\n";
?>