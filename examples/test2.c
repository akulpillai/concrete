#include <stdio.h>

int main() {
    int number;

    printf("Please enter a number: ");
    scanf("%d", &number);

    if (number > 0) {
        printf("You entered a positive number.\n");
    } else if (number < 0) {
        printf("You entered a negative number.\n");
    } else {
        printf("You entered zero.\n");
    }

    return 0;
}

