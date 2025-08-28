import ross_server


def test_schedule():
    sched = ross_server.Schedule(["BA Physics", "BA Chemistry"])
    assert sched.is_valid()
    sched.display()
    sched.save("ross_pytest.xlsx")
    sched2 = ross_server.Schedule.from_file("ross_pytest.xlsx")
    assert sched2.is_valid()
