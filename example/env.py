a = "global"
def block():
    def show_a():
        print(a)
    
    show_a()
    a = "block"
    show_a()

block()

# [line 4] NameError: cannot access free variable 'a' where it is not associated with a value in enclosing scope
