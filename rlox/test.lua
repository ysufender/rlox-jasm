local function main()
    local a = "Hello World"
    local b = 0
    while b < 150000 do
        print(a)
        b = b + 1
    end
end

main()
