public class MethodsAndFields {
    private String name;
    private int age;
    private double salary;

    public MethodsAndFields(String name, int age, double salary) {
        this.name = name;
        this.age = age;
        this.salary = salary;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public int getAge() {
        return age;
    }

    public void setAge(int age) {
        this.age = age;
    }

    public double getSalary() {
        return salary;
    }

    public void setSalary(double salary) {
        this.salary = salary;
    }

    public double calculateBonus(double percentage) {
        return salary * (percentage / 100);
    }
    public static double createAndCalculateBonus(String name, int age, double salary, double percentage) {
        MethodsAndFields employee = new MethodsAndFields(name, age, salary);
        return employee.calculateBonus(percentage);
    }
}