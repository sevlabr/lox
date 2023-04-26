fib :: Int -> Int
fib n | n < 2     = n
      | otherwise = fib (n - 2) + fib (n - 1)

main :: IO()
main = do
    putStrLn "Start"
    putStrLn ("Result: " ++ show (fib 40))
    putStrLn "End"
