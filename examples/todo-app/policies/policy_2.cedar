// Allows more granular user group control, change actions as needed
permit(
    principal in TodoApp::UserGroup::"ENTER_THE_USER_GROUP_HERE",
    action in [
                TodoApp::Action::"assignTask",
        TodoApp::Action::"completeTask",
        TodoApp::Action::"createProject",
        TodoApp::Action::"createTask",
        TodoApp::Action::"deleteProject",
        TodoApp::Action::"deleteTask",
        TodoApp::Action::"getProject",
        TodoApp::Action::"getTask",
        TodoApp::Action::"listProjects",
        TodoApp::Action::"listTasks",
        TodoApp::Action::"updateTask"
    ],
    resource
);